mod routes {
    use rocket_upload::MultipartDatas;
    use tempdir::TempDir;
    use std::path::{Path};
    use rocket::Data;
    use rocket_contrib::json::{Json, JsonValue};
    use futures::executor;
    use rocket::State;

    use crate::cmd::serve::{ClientConfig, DataInfo, DataAddInfo};
    use crate::error::Result;
    use crate::cmd::serve::service::{ipfs_write, ipfs_delete, ipfs_verify, add_order_info};


    #[post("/order/<cid>", data = "<data>")]
    pub(crate) fn create_order(client_config: State<'_, ClientConfig>, cid: Option<String>, data: MultipartDatas) -> Result<JsonValue> {
        let res: Result<JsonValue> = executor::block_on(ipfs_write(client_config.inner(), cid.unwrap().as_str(), data));
        res
    }

    #[post("/order/<cid>/<hash>", format = "json", data = "<data>")]
    pub(crate) fn create_order_info(client_config: State<'_, ClientConfig>, cid: Option<String>, hash: Option<String>, data: Json<DataAddInfo>) -> Result<JsonValue> {
        let res: Result<JsonValue> = executor::block_on(add_order_info(client_config.inner(), cid.unwrap().as_str(), hash, &data.into_inner()));
        res
    }


    #[delete("/order/<cid>/<hash>")]
    pub(crate) fn delete_order(client_config: State<'_, ClientConfig>, cid: Option<String>, hash: Option<String>) -> Result<()> {
        let res: Result<()> = executor::block_on(ipfs_delete(client_config.inner(), cid.unwrap().as_str(), hash));
        res
    }

    #[post("/order/verify/<cid>/<hash>", format = "application/json", data = "<data>")]
    pub(crate) fn verify_order(client_config: State<'_, ClientConfig>, cid: Option<String>, hash: Option<String>, data: Json<DataInfo>) -> Result<JsonValue> {
        let res: Result<JsonValue> = executor::block_on(ipfs_verify(client_config.inner(), cid.unwrap().as_str(), hash, data));
        res
    }
}


mod service {
    use rocket_upload::MultipartDatas;
    use tempdir::TempDir;
    use std::path::Path;
    use ipfs_api::IpfsClient;
    use std::fs::{self, File};
    use std::io::Error;
    use rocket::State;
    use rocket_contrib::json::{Json, JsonValue};

    use crate::cmd::serve::{ClientConfig, DataInfo, DataAddInfo, MinerRequest};
    use crate::error::{Result, MinerError};
    use chrono::{Local, Duration};
    use codec::{Encode, Decode};
    use kvdb::KeyValueDB;

    pub(crate) async fn ipfs_write(client_config: &ClientConfig, cid: &str, data: MultipartDatas) -> Result<JsonValue> {
        let fp = data.files.get(0).ok_or(MinerError::msg("uploading file not found"))?;

        let tmp_dir = TempDir::new(cid)?;


        let file_path = tmp_dir.path().join(format!("{}", cid));

        fp.persist(file_path.as_path());


        // let file_len = fs::metadata(file_path)?.len();
        // limit data file size

        let client = &client_config.ipfs_client;
        let file = File::open(file_path.as_path())?;


        client.files_write(format!("/ipfs/{0}/{1}", cid, fp.filename).as_str(), true, true, file).await?;

        let resp = match client.files_stat(format!("/ipfs/{0}/{1}", cid, fp.filename).as_str()).await {
            Ok(res) => {

                // TODO: write into db
                let ipfs_response = DataInfo {
                    cid: cid.to_string(),
                    name: fp.filename.to_string(),
                    hash: res.hash,
                    size: res.size.to_string(),
                    cumulative_size: res.cumulative_size.to_string(),
                    blocks: res.blocks.to_string(),
                };
                let ipfs_response_encode = ipfs_response.encode();


                let kv_client = &client_config.kv_database.client().unwrap();


                let mut batch = kv_client.transaction();
                // cid-time: ipfs_response{}
                batch.put(0, [cid, ipfs_response.hash.as_str(), Local::now().format("%Y%m%d%H%M%S").to_string().as_str()].concat().as_ref(), ipfs_response_encode.as_ref());

                //save hash data path
                batch.put(2, ipfs_response.hash.as_str().as_ref(), format!("/ipfs/{0}/{1}", cid, ipfs_response.name.as_str()).as_str().as_ref());

                // add data hash pay flag
                batch.put(3, ipfs_response.hash.as_str().as_ref(), b"");

                kv_client.write(batch).unwrap();

                Ok(json!(&ipfs_response))
            }
            Err(e) => Err(MinerError::msg("file not found"))
        };

        tmp_dir.close()?;
        resp
    }

    pub(crate) async fn add_order_info(client_config: &ClientConfig, cid: &str, hash_data: Option<String>, data: &DataAddInfo) -> Result<JsonValue> {
        let kv_client = &client_config.kv_database.client().unwrap();
        let mut res = kv_client.get_by_prefix(0, [cid, hash_data.unwrap().as_str()].concat().as_ref()).unwrap();
        let ipfs_write_decode = DataInfo::decode(&mut &res[..]).unwrap();

        let search_info = MinerRequest {
            cid: ipfs_write_decode.cid.to_string(),
            name: ipfs_write_decode.name.to_string(),
            label: data.label.as_str().to_string(),
            hash: ipfs_write_decode.hash.to_string(),
            category: data.category.as_str().to_string(),
            describe: data.describe.as_str().to_string(),
            size: ipfs_write_decode.size.to_string(),
            add_time: Local::now().format("%Y%m%d%H%M%S").to_string(),
            cumulative_size: ipfs_write_decode.cumulative_size.to_string(),
            blocks: ipfs_write_decode.blocks.to_string(),
        };

        let mut batch = kv_client.transaction();


        let expire_date = Local::now() + Duration::days(data.days as i64);

        // delete data hash pay flag
        batch.delete(3, ipfs_write_decode.hash.as_str().as_ref());

        // save expire info
        batch.put(0, [ipfs_write_decode.hash.as_str(), Local::now().format("%Y%m%d%H%M%S").to_string().as_str()].concat().as_ref(), expire_date.format("%Y%m%d%H%M%S").to_string().as_ref());


        kv_client.write(batch).unwrap();

        // todo: put search_info into search engine

        Ok(json!({ "status": "ok" }))
    }

    pub(crate) async fn ipfs_delete(client_config: &ClientConfig, cid: &str, hash_data: Option<String>) -> Result<()> {
        let client = &client_config.ipfs_client.clone();
        // TODO: select filename from db

        let kv_client = &client_config.kv_database.client().unwrap();

        // let mut res = kv_client.get_by_prefix(0, [cid, hash_data.unwrap().as_str()].concat().as_ref()).unwrap().get(0).unwrap();
        let mut res = kv_client.get_by_prefix(0, [cid, hash_data.unwrap().as_str()].concat().as_ref()).unwrap();

        // let r = &res[..];

        let ipfs_write_decode = DataInfo::decode(&mut &res[..]).unwrap();


        let resp = match client.files_rm(format!("/ipfs/{0}/{1}", cid, ipfs_write_decode.name).as_str(), true).await {
            Ok(res) => Ok(res),
            //TODO:  handling errors
            Err(err) => Err(MinerError::msg("file not found"))
        };
        resp
    }

    pub(crate) async fn ipfs_verify(client_config: &ClientConfig, cid: &str, hash_data: Option<String>, data: Json<DataInfo>) -> Result<JsonValue> {
        let kv_client = &client_config.kv_database.client().unwrap();

        let mut res = kv_client.get_by_prefix(0, [cid, hash_data.unwrap().as_str()].concat().as_ref()).unwrap();
        let res_data = DataInfo::decode(&mut &res[..]).unwrap();

        Ok(json!({"equal":data.0 == res_data}))
    }
}


use rocket_cors;
use rocket_contrib::json::JsonValue;
use rocket_cors::Cors;
use rocket::config::{Config, Environment};

use ipfs_api::{IpfsClient, TryFromUri};
use futures::executor;
use std::io::{self, Read};
use kvdb_rocksdb::{DatabaseConfig, Database};
use http::uri::InvalidUri;
use substrate_subxt::{Client, ClientBuilder};
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use codec::{Encode, Decode};


use crate::settings::{Settings, sub_client, kv_database, ipfs_client};
use crate::storage::kv::rocksdb::KVDatabase;
use crate::constants::META_COL;
use crate::chain::IpseRuntime;
use crate::chain::register_miner;


#[catch(404)]
fn not_found() -> JsonValue {
    json!({
        "status": "error",
        "reason": "Resource was not found."
    })
}

pub(crate) struct ClientConfig {
    sub_client: Client<IpseRuntime>,
    kv_database: KVDatabase,
    ipfs_client: IpfsClient,
}


pub(crate) struct MinerRequest {
    cid: String,
    name: String,
    label: String,
    hash: String,
    category: String,
    describe: String,
    size: String,
    add_time: String,
    cumulative_size: String,
    blocks: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct DataAddInfo {
    label: String,
    category: String,
    describe: String,
    days: u64,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Encode, Decode)]
pub(crate) struct DataInfo {
    cid: String,
    name: String,
    hash: String,
    size: String,
    cumulative_size: String,
    blocks: String,
}


fn cors_fairing() -> Cors {
    Cors::from_options(&Default::default()).expect("Cors fairing cannot be created")
}


pub fn serve(settings: &Settings, address: &str, port: u16) {
    let config = Config::build(Environment::Production)
        .address(address)
        .port(port)
        .finalize().unwrap();

    let client_config = ClientConfig {
        sub_client: sub_client(settings),
        kv_database: kv_database(settings).unwrap(),
        ipfs_client: ipfs_client(settings).unwrap(),
    };

    // register_miner
    executor::block_on(register_miner(settings, sub_client(settings)));

    rocket::custom(config)
        .mount(
            "/api/v0",
            routes![
                routes::create_order,
                routes::create_order_info,
                routes::delete_order,
                routes::verify_order,
            ],
        )
        .manage(client_config)
        .attach(cors_fairing())
        .register(catchers![not_found]).launch();
}
mod routes {
    use tempdir::TempDir;
    use std::path::{Path};
    use rocket::{Request, Data};
    use rocket_contrib::json::{Json, JsonValue};
    use futures::executor;
    use std::fs::{self, File};
    use codec::{Encode, Decode};
    use chrono::{Local, Duration};
    use ubyte::ToByteUnit;

    use rocket::State;

    use crate::cmd::serve::{ClientConfig, DataInfo, DataAddInfo, MinerRequest};
    use crate::util::id::PasteID;
    use crate::error::{Result, MinerError};
    use crate::storage::ipfs::client::Stat;

    // use crate::cmd::serve::service::{ipfs_write, ipfs_delete, ipfs_verify, add_order_info};


    // TODO: 查询用户对应的order_id 和 url -> 链上
    //
    #[get("/")]
    pub(crate) fn hello() -> &'static str {
        "Hello, world!"
    }

    #[post("/order/<cid>", data = "<data>")]
    pub(crate) fn create_order(client_config: State<'_, ClientConfig>, cid: String, data: Data) -> Result<JsonValue> {
        let tmp_dir = TempDir::new(cid.as_str())?;
        let filename = PasteID::rand().to_string();
        let file_path = tmp_dir.path().join(format!("{}", filename));

        println!("{:?}", file_path);

        data.stream_to_file(file_path.clone())
            .map(|n| format!("Wrote {} bytes to {:?}", n, file_path.to_str()));

        let client = &client_config.ipfs_client;

        let resp = client.add(file_path.to_str().unwrap())?;

        let kv_client = &client_config.kv_database.client().unwrap();

        let mut batch = kv_client.transaction();

        let ipfs_response_encode = resp.encode();

        // cid-time: ipfs_response{}
        batch.put(0, [cid.clone(), resp.hash.clone(), Local::now().format("%Y%m%d%H%M%S").to_string()].concat().as_ref(), ipfs_response_encode.as_ref());

        //save hash data path
        batch.put(1, resp.hash.clone().as_str().as_ref(), format!("/ipfs/{0}", cid.clone()).as_str().as_ref());

        // add data hash pay flag
        batch.put(2, resp.hash.clone().as_str().as_ref(), b"");

        kv_client.write(batch).unwrap();


        tmp_dir.close()?;
        // resp
        Ok(json!(resp))
    }

    #[post("/order/<cid>/<hash>", format = "json", data = "<data>")]
    pub(crate) fn create_order_info(client_config: State<'_, ClientConfig>, cid: String, hash: String, data: Json<DataAddInfo>) -> Result<JsonValue> {
        let kv_client = &client_config.kv_database.client().unwrap();
        let mut res = kv_client.get_by_prefix(0, [&cid, hash.as_str()].concat().as_ref()).unwrap();
        let ipfs_write_decode = Stat::decode(&mut &res[..]).unwrap();

        let search_info = MinerRequest {
            cid,
            name: (&data.name).to_string(),
            label: (&data.label).to_string(),
            category: (&data.category).to_string(),
            describe: (&data.describe).to_string(),
            hash,
            st_dev: ipfs_write_decode.st_dev,
            st_ino: ipfs_write_decode.st_ino,
            st_nlink: ipfs_write_decode.st_nlink,
            st_mode: ipfs_write_decode.st_mode,
            st_uid: ipfs_write_decode.st_uid,
            st_gid: ipfs_write_decode.st_gid,
            st_rdev: ipfs_write_decode.st_rdev,
            st_size: ipfs_write_decode.st_size,
            st_blksize: ipfs_write_decode.st_blksize,
            st_blocks: ipfs_write_decode.st_blocks,
            st_atime: ipfs_write_decode.st_atime,
            st_atime_nsec: ipfs_write_decode.st_atime_nsec,
            st_mtime: ipfs_write_decode.st_mtime,
            st_mtime_nsec: ipfs_write_decode.st_mtime_nsec,
            st_ctime: ipfs_write_decode.st_ctime,
            st_ctime_nsec: ipfs_write_decode.st_ctime_nsec,
        };

        let mut batch = kv_client.transaction();


        let expire_date = Local::now() + Duration::days(data.days as i64);

        // delete data hash pay flag
        batch.delete(2, ipfs_write_decode.hash.as_str().as_ref());

        // save expire info
        batch.put(0, [ipfs_write_decode.hash.as_str(), Local::now().format("%Y%m%d%H%M%S").to_string().as_str()].concat().as_ref(), expire_date.format("%Y%m%d%H%M%S").to_string().as_ref());


        kv_client.write(batch).unwrap();

        // todo: put search_info into search engine

        Ok(json!(search_info))
    }


    #[delete("/order/<cid>/<hash>")]
    pub(crate) fn delete_order(client_config: State<'_, ClientConfig>, cid: Option<String>, hash: Option<String>) -> Result<JsonValue> {
        let client = &client_config.ipfs_client;
        // TODO: select filename from db
        let res = client.delete(hash.clone().unwrap().as_str());


        let kv_client = &client_config.kv_database.client().unwrap();

        // let mut res = kv_client.get_by_prefix(0, [cid, hash_data.unwrap().as_str()].concat().as_ref()).unwrap().get(0).unwrap();
        let mut res = kv_client.get_by_prefix(0, [cid.unwrap().as_str(), hash.unwrap().as_str()].concat().as_ref()).unwrap();


        let ipfs_write_decode = Stat::decode(&mut &res[..]).unwrap();


        Ok(json!(ipfs_write_decode))
    }

    #[post("/order/verify/<cid>/<hash>", format = "application/json", data = "<data>")]
    pub(crate) fn verify_order(client_config: State<'_, ClientConfig>, cid: Option<String>, hash: Option<String>, data: Json<DataInfo>) -> Result<JsonValue> {
        let kv_client = &client_config.kv_database.client().unwrap();

        let mut res = kv_client.get_by_prefix(0, [cid.unwrap().as_str(), hash.unwrap().as_str()].concat().as_ref()).unwrap();
        let res_data = DataInfo::decode(&mut &res[..]).unwrap();

        Ok(json!({"equal":data.0 == res_data}))
    }
}


mod service {
    use tempdir::TempDir;
    use std::path::Path;
    use rocket::data::{Data};
    use ubyte::ToByteUnit;

    use std::fs::{self, File};
    use std::io::Error;
    use rocket::{Request, State};
    use rocket_contrib::json::{Json, JsonValue};

    use crate::cmd::serve::{ClientConfig, DataInfo, DataAddInfo, MinerRequest};
    use crate::error::{Result, MinerError};
    use crate::util::id::PasteID;
    use crate::storage::ipfs::client::IpfsClient;

    use chrono::{Local, Duration};
    use codec::{Encode, Decode};
    use kvdb::KeyValueDB;

    // pub(crate) async fn ipfs_write(client_config: &ClientConfig, cid: &str, data: Data) -> Result<JsonValue> {
    //     let tmp_dir = TempDir::new(cid)?;
    //
    //     let filename = PasteID::rand().to_string();
    //
    //     let file_path = tmp_dir.path().join(format!("{}", filename));
    //
    //     data.stream_to_file(file_path.as_path())?;
    //
    //     let client = &client_config.ipfs_client;
    //     let file = File::open(file_path.as_path())?;
    //
    //
    //     client.files_write(format!("/ipfs/{0}/{1}", cid, filename).as_str(), true, true, file).await?;
    //
    //     let resp = match client.files_stat(format!("/ipfs/{0}/{1}", cid, filename).as_str()).await {
    //         Ok(res) => {
    //
    //             // TODO: write into db
    //             let ipfs_response = DataInfo {
    //                 cid: cid.to_string(),
    //                 name: filename.to_string(),
    //                 hash: res.hash,
    //                 size: res.size.to_string(),
    //                 cumulative_size: res.cumulative_size.to_string(),
    //                 blocks: res.blocks.to_string(),
    //             };
    //             let ipfs_response_encode = ipfs_response.encode();
    //
    //
    //             let kv_client = &client_config.kv_database.client().unwrap();
    //
    //
    //             let mut batch = kv_client.transaction();
    //             // cid-time: ipfs_response{}
    //             batch.put(0, [cid, ipfs_response.hash.as_str(), Local::now().format("%Y%m%d%H%M%S").to_string().as_str()].concat().as_ref(), ipfs_response_encode.as_ref());
    //
    //             //save hash data path
    //             batch.put(2, ipfs_response.hash.as_str().as_ref(), format!("/ipfs/{0}/{1}", cid, ipfs_response.name.as_str()).as_str().as_ref());
    //
    //             // add data hash pay flag
    //             batch.put(3, ipfs_response.hash.as_str().as_ref(), b"");
    //
    //             kv_client.write(batch).unwrap();
    //
    //             Ok(json!(&ipfs_response))
    //         }
    //         Err(e) => Err(MinerError::msg("file not found"))
    //     };
    //
    //     tmp_dir.close()?;
    //     resp
    // }

    // pub(crate) async fn add_order_info(client_config: &ClientConfig, cid: &str, hash_data: Option<String>, data: &DataAddInfo) -> Result<JsonValue> {
    //     let kv_client = &client_config.kv_database.client().unwrap();
    //     let mut res = kv_client.get_by_prefix(0, [cid, hash_data.unwrap().as_str()].concat().as_ref()).unwrap();
    //     let ipfs_write_decode = DataInfo::decode(&mut &res[..]).unwrap();
    //
    //     let search_info = MinerRequest {
    //         cid: ipfs_write_decode.cid.to_string(),
    //         name: ipfs_write_decode.name.to_string(),
    //         label: data.label.as_str().to_string(),
    //         hash: ipfs_write_decode.hash.to_string(),
    //         category: data.category.as_str().to_string(),
    //         describe: data.describe.as_str().to_string(),
    //         size: ipfs_write_decode.size.to_string(),
    //         add_time: Local::now().format("%Y%m%d%H%M%S").to_string(),
    //         cumulative_size: ipfs_write_decode.cumulative_size.to_string(),
    //         blocks: ipfs_write_decode.blocks.to_string(),
    //     };
    //
    //     let mut batch = kv_client.transaction();
    //
    //
    //     let expire_date = Local::now() + Duration::days(data.days as i64);
    //
    //     // delete data hash pay flag
    //     batch.delete(3, ipfs_write_decode.hash.as_str().as_ref());
    //
    //     // save expire info
    //     batch.put(0, [ipfs_write_decode.hash.as_str(), Local::now().format("%Y%m%d%H%M%S").to_string().as_str()].concat().as_ref(), expire_date.format("%Y%m%d%H%M%S").to_string().as_ref());
    //
    //
    //     kv_client.write(batch).unwrap();
    //
    //     // todo: put search_info into search engine
    //
    //     Ok(json!({ "status": "ok" }))
    // }
    //
    // pub(crate) async fn ipfs_delete(client_config: &ClientConfig, cid: &str, hash_data: Option<String>) -> Result<()> {
    //     let client = &client_config.ipfs_client.clone();
    //     // TODO: select filename from db
    //
    //     let kv_client = &client_config.kv_database.client().unwrap();
    //
    //     // let mut res = kv_client.get_by_prefix(0, [cid, hash_data.unwrap().as_str()].concat().as_ref()).unwrap().get(0).unwrap();
    //     let mut res = kv_client.get_by_prefix(0, [cid, hash_data.unwrap().as_str()].concat().as_ref()).unwrap();
    //
    //     // let r = &res[..];
    //
    //     let ipfs_write_decode = DataInfo::decode(&mut &res[..]).unwrap();
    //
    //
    //     let resp = match client.files_rm(format!("/ipfs/{0}/{1}", cid, ipfs_write_decode.name).as_str(), true).await {
    //         Ok(res) => Ok(res),
    //         //TODO:  handling errors
    //         Err(err) => Err(MinerError::msg("file not found"))
    //     };
    //     resp
    // }

    // pub(crate) async fn ipfs_verify(client_config: &ClientConfig, cid: &str, hash_data: Option<String>, data: Json<DataInfo>) -> Result<JsonValue> {
    //     let kv_client = &client_config.kv_database.client().unwrap();
    //
    //     let mut res = kv_client.get_by_prefix(0, [cid, hash_data.unwrap().as_str()].concat().as_ref()).unwrap();
    //     let res_data = DataInfo::decode(&mut &res[..]).unwrap();
    //
    //     Ok(json!({"equal":data.0 == res_data}))
    // }
}


use rocket_cors;
use rocket_contrib::json::JsonValue;
use rocket_cors::Cors;
use rocket::config::{Config, Environment};

use futures::executor;
use std::io::{self, Read};
use kvdb_rocksdb::{DatabaseConfig, Database};
use http::uri::InvalidUri;
use substrate_subxt::{Client, ClientBuilder};
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use codec::{Encode, Decode};
use log::{info};


use crate::settings::{Settings, sub_client, kv_database, ipfs_client};
use crate::storage::kv::rocksdb::KVDatabase;
use crate::constants::META_COL;
use crate::chain::IpseRuntime;
use crate::chain::register_miner;
use crate::error::Result;
use crate::storage::ipfs::client::IpfsClient;


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

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Encode, Decode)]
pub(crate) struct MinerRequest {
    // add values
    cid: String,
    name: String,
    label: String,
    category: String,
    describe: String,

    // stat info
    pub hash: String,
    // data hash
    pub st_dev: u64,
    // device number (file system)
    pub st_ino: u64,
    // i-node number (serial number)
    pub st_nlink: u64,
    // number of links
    pub st_mode: u32,
    // file type & mode (permissions)
    pub st_uid: u32,
    // user ID of owner
    pub st_gid: u32,
    // group ID of owner
    pub st_rdev: u64,
    // device number for special files
    pub st_size: i64,
    // size in bytes, for regular files
    pub st_blksize: i64,
    // best I/O block size
    pub st_blocks: i64,
    // number of disk blocks allocated
    pub st_atime: i64,
    // time of last access
    pub st_atime_nsec: i64,
    pub st_mtime: i64,
    // time of last modification
    pub st_mtime_nsec: i64,
    pub st_ctime: i64,
    // time of last file status change
    pub st_ctime_nsec: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct DataAddInfo {
    name: String,
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

pub fn serve(settings: &Settings, address: &str, port: u16) -> Result<()> {
    let config = Config::build(Environment::Production)
        .address(address)
        .port(port)
        .secret_key("7Xui7SN4mI+7egV/9dlfYYLGQJeEx3+DwmSQLwDVXJg=")
        .finalize().unwrap();

    let client_config = ClientConfig {
        sub_client: sub_client(settings)?,
        kv_database: kv_database(settings)?,
        ipfs_client: ipfs_client(settings)?,
    };

    // register_miner
    executor::block_on(register_miner(settings, sub_client(settings)?));

    info!("register_miner");
    println!("register_miner");

    rocket::custom(config)
        .mount(
            "/api/v0",
            routes![
                routes::create_order,
                routes::create_order_info,
                routes::delete_order,
                routes::verify_order,
                routes::hello,
            ],
        )
        .manage(client_config)
        .attach(cors_fairing())
        .register(catchers![not_found]).launch();
    Ok(())
}
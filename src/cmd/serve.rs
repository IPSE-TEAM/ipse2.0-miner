mod routes {
    use tempdir::TempDir;
    use rocket::Data;
    use hex_literal::hex;
    use rocket_contrib::json::{Json, JsonValue};
    use codec::{Encode, Decode};
    use chrono::{Local, Duration};
    use ubyte::ToByteUnit;
    use sp_core::crypto::Ss58Codec;
    use hex as hhex;

    use reqwest;


    use rocket::State;

    use crate::cmd::serve::{ClientConfig, DataInfo, DataAddInfo, MinerRequest};
    use crate::util::id::PasteID;
    use crate::error::{Result, MinerError};
    use crate::storage::ipfs::client::Stat;
    use sp_core::Pair;
    use std::collections::HashMap;

    #[post("/order/<address>", data = "<data>")]
    pub(crate) fn create_order(client_config: State<'_, ClientConfig>, address: String, data: Data) -> Result<JsonValue> {
        let tmp_dir = TempDir::new(address.as_str())?;
        let filename = PasteID::rand().to_string();
        let file_path = tmp_dir.path().join(format!("{}", filename));

        data.stream_to_file(file_path.clone())
            .map(|n| format!("Wrote {} bytes to {:?}", n, file_path.to_str()))?;

        let client = &client_config.ipfs_client;

        let resp = client.add(file_path.to_str()?)?;


        let kv_client = &client_config.kv_database.client()?;

        let mut batch = kv_client.transaction();

        let ipfs_response_encode = resp.encode();

        // address-time: ipfs_response{}
        batch.put(0, [address.clone(), resp.hash.clone(), Local::now().format("%Y%m%d%H%M%S").to_string()].concat().as_ref(), ipfs_response_encode.as_ref());

        //save hash data path
        // 插入数据，如果是 1 表示新插入数据
        batch.put(1, resp.hash.clone().as_str().as_ref(), b"1");


        // add data hash pay flag
        batch.put(2, resp.hash.clone().as_str().as_ref(), b"");

        kv_client.write(batch)?;

        tmp_dir.close()?;
        Ok(json!(resp))
    }

    #[post("/order/<address>/<hash>", format = "json", data = "<data>")]
    pub(crate) fn create_order_info(client_config: State<'_, ClientConfig>, address: String, hash: String, data: Json<DataAddInfo>) -> Result<JsonValue> {
        let kv_client = &client_config.kv_database.client()?;
        let settings = &client_config.settings;

        let res = kv_client.get_by_prefix(0, [&address, hash.to_owned().as_str()].concat().as_ref())?;
        let ipfs_write_decode = Stat::decode(&mut &res[..])?;

        let pair = &client_config.pair;



        let signature = pair.sign(&hash.to_owned().as_bytes());

        let search_info = MinerRequest {
            address,
            name: (&data.name).to_string(),
            label: (&data.label).to_string(),
            category: (&data.category).to_string(),
            describe: (&data.describe).to_string(),
            hash,
            sig: hhex::encode(signature.0),
            public_key: settings.to_owned().miner.public_key,
            st_dev: ipfs_write_decode.st_dev,
            st_ino: ipfs_write_decode.st_ino,
            st_nlink: ipfs_write_decode.st_nlink,
            st_mode: ipfs_write_decode.st_mode,
            st_uid: ipfs_write_decode.st_uid,
            st_gid: ipfs_write_decode.st_gid,
            st_rdev: ipfs_write_decode.st_rdev,
            size: ipfs_write_decode.st_size,
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

        let client = reqwest::blocking::Client::new();
        let res = client.post(&settings.search.url)
            .json(&search_info)
            .send()?;

        println!("请求的json: {:?}", &search_info);

        let expire_date = Local::now() + Duration::days(data.days as i64);

        // delete data hash pay flag
        batch.delete(2, ipfs_write_decode.hash.as_str().as_ref());

        // save expire info
        batch.put(0, [ipfs_write_decode.hash.as_str(), Local::now().format("%Y%m%d%H%M%S").to_string().as_str()].concat().as_ref(), expire_date.format("%Y%m%d%H%M%S").to_string().as_ref());


        kv_client.write(batch)?;

        // todo: put search_info into search engine

        Ok(json!(search_info))
    }


    #[delete("/order/<address>/<hash>")]
    pub(crate) fn delete_order(client_config: State<'_, ClientConfig>, address: String, hash: String) -> Result<JsonValue> {
        let client = &client_config.ipfs_client;
        // TODO: select filename from db
        let res = client.delete(hash.to_owned().as_str());


        let kv_client = &client_config.kv_database.client()?;

        // let mut res = kv_client.get_by_prefix(0, [address, hash_data.unwrap().as_str()].concat().as_ref()).unwrap().get(0).unwrap();
        let res = kv_client.get_by_prefix(0, [address.as_str(), hash.to_owned().as_str()].concat().as_ref())?;


        let ipfs_write_decode = Stat::decode(&mut &res[..])?;


        Ok(json!(ipfs_write_decode))
    }

    #[post("/order/verify/<address>/<hash>", format = "application/json", data = "<data>")]
    pub(crate) fn verify_order(client_config: State<'_, ClientConfig>, address: String, hash: String, data: Json<DataInfo>) -> Result<JsonValue> {
        let kv_client = &client_config.kv_database.client()?;

        let res = kv_client.get_by_prefix(0, [address.as_str(), hash.as_str()].concat().as_ref())?;
        let res_data = DataInfo::decode(&mut &res[..])?;

        Ok(json!({"equal":data.0 == res_data}))
    }
}


use rocket_cors;
use rocket_contrib::json::JsonValue;
use rocket_cors::Cors;
use rocket::config::{Config, Environment};

use futures::executor;
use std::io::{self, Read};
use kvdb_rocksdb::{DatabaseConfig, Database};
use substrate_subxt::Client;
use serde::{Deserialize, Serialize};
use codec::{Encode, Decode};
use sp_core::{sr25519::Pair, Pair as PairT};


use crate::settings::{Settings, sub_client, kv_database, ipfs_client};
use crate::storage::kv::rocksdb::KVDatabase;
use crate::chain::IpseRuntime;
use crate::chain::register_miner;
use crate::error::Result;
use crate::storage::ipfs::client::IpfsClient;
use hex as hhex;


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
    settings: Settings,
    pair: Pair,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Encode, Decode)]
pub(crate) struct MinerRequest {
    // add values
    address: String,
    name: String,
    label: String,
    category: String,
    describe: String,
    public_key: String,

    // stat info
    pub hash: String,
    // sig info
    pub sig: String,
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
    // pub st_size: i64,
    pub size: i64,
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
    address: String,
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
    // let config = Config::build(Environment::Production)
    let config = Config::build(Environment::Development)
        .address(address)
        .port(port)
        .secret_key("7Xui7SN4mI+7egV/9dlfYYLGQJeEx3+DwmSQLwDVXJg=")
        .finalize()?;


    let seed = settings.miner.secret_seed.as_str();
    let pair = Pair::from_seed_slice(&hhex::decode(&seed[..])?)?;

    let client_config = ClientConfig {
        sub_client: sub_client(settings)?,
        kv_database: kv_database(settings)?,
        ipfs_client: ipfs_client(settings)?,
        settings: settings.to_owned(),
        pair: pair.to_owned(),
    };

    // register_miner
    executor::block_on(register_miner(settings, pair.to_owned(), sub_client(settings)?))?;


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
    Ok(())
}
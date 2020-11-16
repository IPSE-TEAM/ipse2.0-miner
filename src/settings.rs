use std::env;
use config::{ConfigError, Config, File, Environment};
use std::collections::HashMap;
use lazy_static::lazy_static;
use substrate_subxt::{Client, ClientBuilder};
use futures::executor;
use std::io;
use kvdb_rocksdb::DatabaseConfig;
use ipfs_api::{IpfsClient, TryFromUri};
use http::uri::InvalidUri;
use std::path::PathBuf;
use crate::chain::IpseRuntime;
use crate::storage::kv::rocksdb::KVDatabase;
use crate::constants::META_COL;


#[derive(Debug, Deserialize, Clone)]
pub struct Miner {
    pub nickname: String,
    pub region: String,
    pub url: String,
    pub capacity: u64,
    pub unit_price: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Chain {
    pub url: String,
}


#[derive(Debug, Deserialize, Clone)]
pub struct Data {
    pub db: String,
    pub keystore: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Ipfs {
    pub uri: String,
    pub local: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub miner: Miner,
    pub chain: Chain,
    pub data: Data,
    pub ipfs: Ipfs,
}


impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        // Start off by merging in the "default" configuration file
        s.merge(File::with_name("config/default"))?;

        // Add in the current environment file
        // Default to 'production' env
        // Note that this file is _optional_
        let env = env::var("RUN_MODE").unwrap_or_else(|_| "production".into());
        s.merge(File::with_name(&format!("config/{}", env)).required(false))?;

        s.try_into()
    }
    pub fn build(file: PathBuf) -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name(file.into_os_string().into_string().unwrap().as_str()))?;
        s.try_into()
    }
}


pub fn sub_client(settings: &Settings) -> Client<IpseRuntime> {
    let chain_url = settings.chain.url.clone();

    let res = executor::block_on(
        ClientBuilder::<IpseRuntime>::new()
            .set_url(chain_url)
            .build()
    );
    res.unwrap()
}

pub fn kv_database(settings: &Settings) -> io::Result<KVDatabase> {
    let config = DatabaseConfig::with_columns(META_COL);

    let path = settings.data.db.clone();

    Ok(KVDatabase {
        config,
        path,
    })
}


pub fn ipfs_client(settings: &Settings) -> Result<IpfsClient, InvalidUri> {
    IpfsClient::from_str(settings.ipfs.uri.as_str())
}
use std::env;
use config::{ConfigError, Config, File, Environment};
use std::collections::HashMap;
use substrate_subxt::{Client, ClientBuilder};
use futures::executor;
use std::io;
use kvdb_rocksdb::DatabaseConfig;
use std::path::PathBuf;
use std::result;
use crate::chain::IpseRuntime;
use crate::storage::kv::rocksdb::KVDatabase;
use crate::constants::META_COL;
use crate::error::{Result, MinerError};
use crate::storage::ipfs::client::IpfsClient;


#[derive(Debug, Deserialize, Clone)]
pub struct Miner {
    pub nickname: String,
    pub region: String,
    pub url: String,
    pub public_key: String,
    pub secret_seed: String,
    pub income_address: String,
    pub capacity: u64,
    pub unit_price: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Chain {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Search {
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
    pub search: Search,
}


impl Settings {
    pub fn new() -> result::Result<Self, ConfigError> {
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
    pub fn build(file: PathBuf) -> result::Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name(file.into_os_string().into_string().unwrap().as_str()))?;
        s.try_into()
    }
}


pub fn sub_client(settings: &Settings) -> Result<Client<IpseRuntime>> {
    let chain_url = settings.chain.url.clone();

    executor::block_on(
        ClientBuilder::<IpseRuntime>::new()
            .set_url(chain_url)
            .build()
    ).map_err(|_| MinerError::msg("ipse server connect error"))
}

pub fn kv_database(settings: &Settings) -> Result<KVDatabase> {
    let config = DatabaseConfig::with_columns(META_COL);

    let path = settings.data.db.clone();

    Ok(KVDatabase {
        config,
        path,
    })
}


pub fn ipfs_client(settings: &Settings) -> Result<IpfsClient> {
    Ok(IpfsClient::new(settings.ipfs.uri.as_str()))
}
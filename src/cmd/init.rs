use std::fs::{create_dir_all};
use std::path::Path;
use std::path::PathBuf;

use crate::error::Result;
use crate::utils::create_file;
use crate::error::MinerError;
use crate::settings::Settings;

const CONFIG: &str = r#"[miner]
nickname = "the_name_of_miner"
region = "the_regin_of_miner"
url = "http://localhost:7001"
capacity = 1024000000
unit_price = 100

[chain]
url = "ws://localhost:9944"

[data]
db = "db"
keystore = "keystore"


[ipfs]
uri = "http://127.0.0.1:5001"
local = false
"#;


pub fn is_directory_empty(path: &Path) -> Result<bool> {
    if path.is_dir() {
        let is_entry = match path.read_dir() {
            Ok(entries) => Ok(false),
            Err(e) => Ok(true)
        };
        return is_entry;
    }
    Ok(false)
}

pub fn init(name: &str, force: bool) -> Result<()> {
    let path = Path::new(name);

    // TODO: Provide more tips
    if path.exists() && !is_directory_empty(&path)? && !force {
        return if name == "." {
            Err(MinerError::msg(format!("Failed to create current path(.), please init for other name, for example: miner")))
        } else {
            Err(MinerError::msg(format!(
                "`{}` is not an empty folder (hidden files are ignored).",
                path.to_string_lossy().to_string()
            )))
        };
    }

    // generate project data catalog
    create_file(&path.join("config.toml"), &CONFIG)?;
    create_dir_all(path.join("db"))?;
    create_dir_all(path.join("keystore"))?;

    Ok(())
}

pub fn init_db(settings: &Settings) {
    // TODO: Don't create it at init
}


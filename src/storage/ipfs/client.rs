use reqwest;
use failure::Error;

use serde::{Deserialize, Serialize};
use codec::{Encode, Decode};
use std::io::Read;
use reqwest::multipart::Part;
use serde_json;
use reqwest::Client;

use std::fs;
use nix::sys::stat::stat;
use std::path::Path;

use crate::error::{MinerError, Result};

#[derive(Clone)]
pub struct IpfsClient {
    uri: String,
}


#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Encode, Decode)]
pub struct Stat {
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


impl Default for IpfsClient {
    fn default() -> Self {
        Self { uri: "127.0.0.1:5001".parse().unwrap() }
    }
}


impl IpfsClient {
    /// ipfs http api,
    /// https://docs.ipfs.io/reference/http/api/
    pub fn new(uri: &str) -> IpfsClient {
        IpfsClient {
            uri: uri.to_string(),
        }
    }

    pub fn uri(&self) -> String {
        format!("{}", self.uri)
    }

    pub fn add(&self, data: &str) -> Result<Stat> {
        let (code, stdout, stderr) = sh!("ipfs add  {}", data);


        if &stderr == "" {
            return Err(MinerError::msg("serve add file error"));
        }

        let filename = Path::new(data);

        let mut iter = stdout.split_whitespace();
        iter.next();
        let file_hash = iter.next().unwrap();

        let stat_result = stat(filename).unwrap();

        Ok(Stat {
            hash: file_hash.to_string(),
            st_dev: stat_result.st_dev,
            st_ino: stat_result.st_ino,
            st_nlink: stat_result.st_nlink,
            st_mode: stat_result.st_mode,
            st_uid: stat_result.st_uid,
            st_gid: stat_result.st_gid,
            st_rdev: stat_result.st_rdev,
            st_size: stat_result.st_size,
            st_blksize: stat_result.st_blksize,
            st_blocks: stat_result.st_blocks,
            st_atime: stat_result.st_atime,
            st_atime_nsec: stat_result.st_atime_nsec,
            st_mtime: stat_result.st_mtime,
            st_mtime_nsec: stat_result.st_mtime_nsec,
            st_ctime: stat_result.st_ctime,
            st_ctime_nsec: stat_result.st_ctime_nsec,
        })
    }

    pub fn delete(&self, hash: &str) -> (i32, String, String) {
        sh!("ipfs pin rm  {} && ipfs repo gc", hash)
    }
}

#[cfg(test)]
mod test {
    use crate::storage::ipfs::client::IpfsClient;

    #[test]
    fn test_default_client() {
        let client = IpfsClient::default();
        let uri = client.uri();
        assert_eq!("http://127.0.0.1:5001/", uri);
    }
}


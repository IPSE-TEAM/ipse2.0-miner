use reqwest;
use failure::Error;

use std::io::Read;
use reqwest::multipart::Part;
use serde_json;
use reqwest::Client;

#[derive(Clone)]
pub struct IpfsClient {
    server: String,
    port: u16,
}


impl Default for IpfsClient {
    fn default() -> Self {
        Self { server: "127.0.0.1".parse().unwrap(), port: 5001 }
    }
}


impl IpfsClient {
    /// ipfs http api,
    /// https://docs.ipfs.io/reference/http/api/
    pub fn new(server: &str, port: u16) -> IpfsClient {
        IpfsClient {
            server: server.into(),
            port,
        }
    }

    pub fn url(&self) -> String {
        format!("http://{}:{}/", self.server, self.port)
    }



}

#[cfg(test)]
mod test {
    use crate::storage::ipfs::client::IpfsClient;

    #[test]
    fn test_default_client() {
        let client = IpfsClient::default();
        let url = client.url();
        assert_eq!("http://127.0.0.1:5001/", url);
    }
}


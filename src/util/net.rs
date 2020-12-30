use std::io::Read;
use reqwest::StatusCode;
use reqwest::blocking::Response;


use crate::error::Result;


#[derive(Debug, Clone)]
pub struct Client {
    host: String,
}

impl Client {
    pub fn new(host: String) -> Self {
        Client {
            host
        }
    }

    // pub fn post(&self, endpoint: &str) -> Result<String> {
    //     let url: String = format!("{}{}", self.host, endpoint);
    //
    //     let client = reqwest::blocking::Client::new();
    //     let response = client
    //         .post(url.as_str())
    //         // .headers(self.build_headers(false)?)
    //         .send()?;
    //
    //     self.handler(response)
    // }
    //
    //
    // fn handler(&self, mut response: Response) -> Result<String> {
    //     match response.status() {
    //         StatusCode::OK => {
    //             let mut body = String::new();
    //             response.read_to_string(&mut body)?;
    //             Ok(body)
    //         }
    //     }
    // }
}

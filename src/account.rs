use std::fs;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use serde_json::json as sjson;
use std::time::SystemTime;

use crate::crypto::*;
use crate::pkcs8;


#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Default)]
pub struct Address {
    pub addr: String,
    pub label: String,
    pub crypto_type: String,
    pub seed: Vec<u8>,
    pub created_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Encoding {
    pub content: Vec<String>,
    pub r#type: String,
    pub version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Keystore {
    pub address: String,
    pub encoded: String,
    pub encoding: Encoding,
    pub meta: Value,
}

impl Keystore {
    pub fn parse_from_file(path: String) -> Result<Self, ()> {
        let data = fs::read_to_string(path).map_err(|_| ())?;
        let keystore: Self = serde_json::from_str(&data).map_err(|_| ())?;
        Ok(keystore)
    }

    pub fn crypto(&self) -> String {
        self.encoding.content[1].clone()
    }

    pub fn label(&self) -> String {
        self.meta["name"].as_str().unwrap_or("").to_string()
    }

    pub fn genesis_hash(&self) -> String {
        self.meta["genesisHash"].as_str().unwrap_or("").to_string()
    }

    pub fn when_created(&self) -> u64 {
        self.meta["whenCreated"].as_u64().unwrap_or(0u64)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn encoded_bytes(&self) -> Vec<u8> {
        let encoded = if self.encoded.starts_with("0x") {
            &self.encoded[2..]
        } else {
            &self.encoded
        };
        hex::decode(encoded).unwrap_or(vec![])
    }

    pub fn into_pair<T: Crypto>(&self, password: Option<String>) -> Result<T::Pair, ()> {
        let encoded = self.encoded_bytes();
        if encoded.is_empty() {
            return Err(());
        }
        match pkcs8::decode(&encoded[..], password) {
            Ok((_, secret_key)) => {
                T::pair_from_secret_slice(&secret_key[..])
            }
            Err(_) => Err(())
        }
    }
}


impl Address {
    pub fn into_keystore(&self, password: Option<String>) -> Keystore {
        let mut keystore = Keystore {
            address: self.addr.clone(),
            encoded: "".to_string(),
            encoding: Encoding {
                content: vec!["pkcs8".to_owned()],
                r#type: "xsalsa20-poly1305".to_owned(),
                version: "2".to_owned(),
            },
            meta: sjson!({
                "name": self.label,
                "whenCreated": self.created_at,
              }),
        };

        let (public_key, secret_key) = match self.crypto_type.as_str() {
            "sr25519" => {
                let pair = Sr25519::pair_from_secret_slice(&self.seed[..]).unwrap();
                (pair.public().to_raw_vec(), pair.to_raw_vec())
            }
            "ed25519" => {
                let pair = Ed25519::pair_from_secret_slice(&self.seed[..]).unwrap();
                (pair.public().to_raw_vec(), pair.to_raw_vec())
            }
            "ecdsa" => {
                let pair = Ecdsa::pair_from_secret_slice(&self.seed[..]).unwrap();
                (pair.public().to_raw_vec(), pair.to_raw_vec())
            }
            _ => unreachable!()
        };

        let encoded = pkcs8::encode(&secret_key[..], &public_key[..], password).unwrap();
        keystore.encoded = format!("0x{}", hex::encode(encoded));
        keystore.encoding.content.push(self.crypto_type.clone());
        keystore
    }
    pub fn from_keystore(keystore: Keystore, password: Option<String>) -> Result<Self, ()> {
        let mut address = Self::default();
        address.label = keystore.label();
        address.created_at = keystore.when_created();
        address.crypto_type = keystore.crypto().clone();

        match keystore.crypto().as_str() {
            "ecdsa" => {
                if let Ok(pair) = keystore.into_pair::<Ecdsa>(password) {
                    address.addr = Ecdsa::to_address(&pair);
                    address.seed = pair.to_raw_vec();
                } else {
                    return Err(());
                }
            }
            "sr25519" => {
                if let Ok(pair) = keystore.into_pair::<Sr25519>(password) {
                    address.addr = Sr25519::to_address(&pair);
                    address.seed = pair.to_raw_vec();
                } else {
                    return Err(());
                }
            }
            "ed25519" => {
                if let Ok(pair) = keystore.into_pair::<Ed25519>(password) {
                    address.addr = Ed25519::to_address(&pair);
                    address.seed = pair.to_raw_vec();
                } else {
                    return Err(());
                }
            }
            _ => {
                return Err(());
            }
        }

        Ok(address)
    }

    pub fn generate<T: Crypto>() -> Self {
        let (pair, _, seed) = T::Pair::generate_with_phrase(None);
        // [u8; 8] to &[u8]
        let seed_slice: &[u8] = seed.as_ref();
        let addr = T::to_address(&pair);
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64;
        Address {
            label: String::default(),
            addr,
            crypto_type: T::crypto_type().to_owned(),
            seed: seed_slice.to_vec(),
            created_at: now,
        }
    }

    pub fn from_phrase<T: Crypto>(phrase: &str) -> Result<Self, ()> {
        match T::Pair::from_phrase(phrase, None) {
            Ok((pair, seed)) => {
                println!("--------------{:?}", seed.as_ref());

                let seed_slice: &[u8] = seed.as_ref();
                let addr = T::to_address(&pair);
                let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64;
                let address = Address {
                    label: String::default(),
                    addr,
                    crypto_type: T::crypto_type().to_owned(),
                    seed: seed_slice.to_vec(),
                    created_at: now,
                };
                Ok(address)
            }
            Err(_) => return Err(()),
        }
    }

    // pub fn from_seed<T: Crypto>(seed: &[u8]) -> Result<Self, ()> {
    //     match T::Pair::from_seed_slice(seed) {
    //         Ok((pair)) => {
    //             let addr = T::to_address(&pair);
    //             let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64;
    //             let address = Address {
    //                 label: String::default(),
    //                 addr,
    //                 crypto_type: T::crypto_type().to_owned(),
    //                 seed: seed.to_vec(),
    //                 created_at: now,
    //             };
    //             Ok(address)
    //         }
    //         Err(_) => return Err(()),
    //     }
    // }

    pub fn into_pair<T: Crypto>(&self) -> <T as Crypto>::Pair {
        T::pair_from_secret_slice(&self.seed[..]).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::str;
    use super::*;
    use hex_literal::hex;
    use sp_core::{ed25519, sr25519, ecdsa, Pair};


    // #[test]
    // fn test_from_keystore_for_sr25519() {
    //     let seed = hex!("e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a");
    //     let pair = sr25519::Pair::from_seed(&seed);
    //
    //     let expect_address = Address {
    //         addr: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_owned(),
    //         label: "sr25519".to_owned(),
    //         crypto_type: "sr25519".to_owned(),
    //         seed: pair.to_raw_vec(),
    //         created_at: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64,
    //     };
    //
    //     println!("expect_address: {:?} \n", expect_address);
    //
    //
    //     let address = Address::from_phrase::<Sr25519>(&seed).unwrap();
    //     println!("address: {:?} \n", address);
    //     assert_eq!(address, expect_address);
    // }


    // #[test]
    // fn test_from_phrase_for_sr25519() {
    //     use std::str;
    //
    //     let phrase = "parade critic curious route discover napkin kitchen clump steel scorpion fat crumble";
    //
    //
    //     let expect_address = Address {
    //         addr: "5GjtRYLYB3Seq7q8zZZ34T7L3ttX6MwJ9B9ZwC6i5cdYLZE6".to_owned(),
    //         label: "".to_owned(),
    //         crypto_type: "sr25519".to_owned(),
    //         seed: b"750274825104bedadf2bdd2e6dfae92c85d8ebcb4d4e9ed8a25ac8fd60dde19f".to_vec(),
    //         created_at: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64,
    //     };
    //
    //
    //     let address = Address::from_phrase::<Sr25519>(phrase).unwrap();
    //
    //
    //     println!("expect_address.seed: {:?}", str::from_utf8(&expect_address.seed[..]));
    //     println!("address.seed: {:?}", str::from_utf8(&address.seed[..]));
    //
    //     println!("\n \n");
    //
    //     println!("expect_address: {:?} \n", expect_address);
    //     println!("address: {:?} \n", address);
    //
    //
    //     assert_eq!(address, expect_address);
    // }

    #[test]
    fn test_from_seed_for_sr25519() {
        let seed = hex!("750274825104bedadf2bdd2e6dfae92c85d8ebcb4d4e9ed8a25ac8fd60dde19f");
        let pair = sr25519::Pair::from_seed(&seed);

        let address = Address {
            addr: pair.public().to_ss58check(),
            label: "sr25519".to_owned(),
            crypto_type: "sr25519".to_owned(),
            seed: pair.to_raw_vec(),
            created_at: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64,
        };

        let expect_address = Address {
            addr: "5GjtRYLYB3Seq7q8zZZ34T7L3ttX6MwJ9B9ZwC6i5cdYLZE6".to_owned(),
            label: "sr25519".to_owned(),
            crypto_type: "sr25519".to_owned(),
            seed: b"750274825104bedadf2bdd2e6dfae92c85d8ebcb4d4e9ed8a25ac8fd60dde19f".to_vec(),
            created_at: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64,
        };

        println!("expect_address.seed: {:?}", str::from_utf8(&expect_address.seed[..]));

        println!("address.seed: {:?}", str::from_utf8(&address.seed[..]));

        println!("address: {:?} \n", address);
        println!("expect_address: {:?} \n", expect_address);

        assert_eq!(address, expect_address);
    }
}
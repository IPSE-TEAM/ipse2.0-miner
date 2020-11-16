pub mod rocksdb;

#[derive(Debug)]
pub struct MinerDbKey {
    pub module: String,
    pub time: String,
    pub cid: String,
    pub hash: String,
}

impl MinerDbKey {
    pub fn new(&self, module: String, time: String, cid: String, hash: String) -> Self {
        Self {
            module,
            time,
            cid,
            hash,
        }
    }


    pub fn value(&self) -> String {
        [self.module.as_str(), self.time.as_str(), self.cid.as_str(), self.hash.as_str()].concat()
    }

    pub fn build_for_time(time: String, cid: String, hash: String) -> Self {
        MinerDbKey {
            module: "".to_string(),
            time,
            cid,
            hash,
        }
    }

    pub fn build_for_cid(cid: String, hash: String) -> Self {
        MinerDbKey {
            module: "".to_string(),
            time: "".to_string(),
            cid,
            hash,
        }
    }

    pub fn build_for_hash(cid: String, hash: String) -> Self {
        MinerDbKey {
            module: "".to_string(),
            time: "".to_string(),
            cid: "".to_string(),
            hash,
        }
    }
}
pub mod rocksdb;

#[derive(Debug)]
pub struct MinerDbKey {
    pub module: String,
    pub time: String,
    pub address: String,
    pub hash: String,
}

impl MinerDbKey {
    pub fn new(&self, module: String, time: String, address: String, hash: String) -> Self {
        Self {
            module,
            time,
            address,
            hash,
        }
    }


    pub fn value(&self) -> String {
        [self.module.as_str(), self.time.as_str(), self.address.as_str(), self.hash.as_str()].concat()
    }
}
use serde::Deserialize;


pub struct DiskInfo {
    pub free_space: u64,
    pub fstype: String,
    pub total_space: u64,
}


pub struct Memory {
    pub memory: u64,
    pub virt: u64,
}

pub struct Runtime {
    pub arch: String,
    pub compiler: String,
    pub gomaxprocs: u64,
    pub numcpu: u64,
    pub numgoroutines: u64,
    pub os: String,
    pub version: String,
}
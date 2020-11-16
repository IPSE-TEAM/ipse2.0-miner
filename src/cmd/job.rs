/// Scheduling tasks  for miner
use job_scheduler::{JobScheduler, Job};

use crate::settings::{Settings, kv_database, ipfs_client};
use kvdb::KeyValueDB;
use chrono::{Local};
use std::str;
use futures::executor;
use std::time::Duration;
use std::error::Error;


pub fn delete_by_hash(settings: &Settings, hash: &[u8]) {
    let kv_client = kv_database(settings).unwrap().client().unwrap();
    let path = kv_client.get(2, hash);

    let ic = ipfs_client(settings).unwrap();
    executor::block_on(ic.files_rm(str::from_utf8(&path.unwrap().unwrap()).unwrap(), true));
    ()
}


pub fn rm_expired_data(settings: &Settings) {
    let kv_client = kv_database(settings).unwrap().client().unwrap();

    let select_key = Local::now().format("%Y%m%d%H%M%S").to_string();

    kv_client.iter(1).filter(move |(i, x)|
        x.starts_with(select_key.as_ref())
    ).for_each(|(j, y)| delete_by_hash(settings, y.as_ref()));
    ()
}


pub fn job(settings: &Settings) {
    let mut sched = JobScheduler::new();


    // Execute function every morning
    sched.add(Job::new("0 0 0 * * ?".parse().unwrap(), || {
        println!("start rm expired data file");
        rm_expired_data(settings);
        println!("end rm expired data file");
    }));

    loop {
        sched.tick();

        std::thread::sleep(Duration::from_millis(1000));
    }
}
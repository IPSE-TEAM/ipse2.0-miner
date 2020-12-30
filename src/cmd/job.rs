/// Scheduling tasks  for miner
use job_scheduler::{JobScheduler, Job};

use crate::settings::{Settings, kv_database, ipfs_client};
use kvdb::KeyValueDB;
use chrono::{Local};
use std::str;
use futures::executor;
use std::time::Duration;
use log::{self, LevelFilter};

use crate::error::Result;


pub fn delete_by_hash(settings: &Settings, hash: &[u8]) {
    let kv_client = kv_database(settings).unwrap().client().unwrap();
    let ipfs_client = ipfs_client(settings).unwrap();
    ipfs_client.delete(str::from_utf8(hash).unwrap());


    // let ic = ipfs_client(settings).unwrap();
    // executor::block_on(ic.files_rm(str::from_utf8(&path.unwrap().unwrap()).unwrap(), true));
    ()
}

/// update miner info(capacity)
// pub fn update_miner_info(settings: &Settings) {
//
//     ()
// }

pub fn rm_expired_data(settings: &Settings) {
    let kv_client = kv_database(settings).unwrap().client().unwrap();

    let select_key = Local::now().format("%Y%m%d%H%M%S").to_string();

    kv_client.iter(1).filter(move |(i, x)|
        x.starts_with(select_key.as_ref())
    ).for_each(|(j, y)| delete_by_hash(settings, y.as_ref()));
    ()
}


pub fn job(settings: &Settings) ->Result<()> {
    let mut sched = JobScheduler::new();
    log::info!("start miner job");

    // Execute function every morning
    sched.add(Job::new("10 * * * * *".parse().unwrap(), || {
        log::info!("start miner job");
        println!("start rm expired data file");
        rm_expired_data(settings);
        println!("end rm expired data file");
    }));

    loop {
        sched.tick();

        std::thread::sleep(Duration::from_millis(1000));
    }
    Ok(())
}
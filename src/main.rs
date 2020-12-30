#![feature(backtrace)]
#![feature(proc_macro_hygiene, decl_macro)]
#![feature(try_trait)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate shells;

use chrono::Local;
use std::env;
use std::path::PathBuf;
use std::path::Path;
use env_logger::Builder;
use log::{self, LevelFilter};


use crate::cmd::{serve, init, job, generate};
use crate::settings::Settings;
use crate::error::log_backtrace;


mod cli;
mod cmd;
mod constants;
mod error;
mod settings;
mod storage;
mod chain;
mod util;
mod color;


fn main() {
    env_logger::init();
    let matches = cli::build_cli().get_matches();

    let root_dir = match matches.value_of("root").unwrap() {
        "." => env::current_dir().unwrap(),
        path => {
            let mut return_path = PathBuf::from(path);

            if !Path::new(path).is_absolute() {
                return_path = env::current_dir().unwrap().join(path);
            }
            return_path
                .canonicalize()
                .unwrap_or_else(|_| panic!("Cannot find root directory: {}", path))
        }
    };
    let config_file = match matches.value_of("config") {
        Some(path) => PathBuf::from(path),
        None => root_dir.join("config.toml"),
    };

    let res = match matches.subcommand() {
        ("serve", Some(matches)) => {
            let address = matches.value_of("address").unwrap_or_default();
            let port = matches.value_of("port").unwrap_or_default().parse::<u16>().unwrap();

            // load setting for current fold
            let settings = Settings::build(config_file).unwrap();

            serve(&settings, address, port)
        }
        ("init", Some(matches)) => {
            let force = matches.is_present("force");
            cmd::init(matches.value_of("name").unwrap(), force)
        }
        ("generate", Some(matches)) => {
            let words = matches.value_of("words").unwrap_or_default();
            generate(words)
        }
        ("job", Some(matches)) => {
            let settings = Settings::build(config_file).unwrap();
            job(&settings)
        }
        _ => unreachable!(),
    };
    if let Err(e) = res {
        log::error!("Error: {}", e);

        std::process::exit(101);
    }
}
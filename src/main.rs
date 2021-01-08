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

use yansi::Paint;
use chrono::Local;
use std::env;
use std::path::PathBuf;
use std::path::Path;
use log::{self, LevelFilter};

use std::io::Write;
use env_logger::{fmt::Color, Builder, Env};


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


fn init_logger() {
    let mut builder = Builder::new();

    builder.format(|formatter, record| {
        let mut style = formatter.style();
        style.set_bold(true);

        let tar = Paint::blue("Miner Serve").bold();

        match record.level() {
            log::Level::Info => writeln!(
                formatter,
                "{} {} ({}): {}",
                tar,
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                Paint::blue("Info").bold(),
                Paint::blue(record.args()).wrap()
            ),
            log::Level::Trace => writeln!(
                formatter,
                "{} {} ({}): {}",
                tar,
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                Paint::magenta("Trace").bold(),
                Paint::magenta(record.args()).wrap()
            ),
            log::Level::Error => writeln!(
                formatter,
                "{} {} ({}): {}",
                tar,
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                Paint::red("Error").bold(),
                Paint::red(record.args()).wrap()
            ),
            log::Level::Warn => writeln!(
                formatter,
                "{} {} ({}): {}",
                tar,
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                Paint::yellow("Warning").bold(),
                Paint::yellow(record.args()).wrap()
            ),
            log::Level::Debug => writeln!(
                formatter,
                "{} {} ({}): {}",
                tar,
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                Paint::blue("Debug").bold(),
                Paint::blue(record.args()).wrap()
            ),
        }
    });

    if let Ok(var) = env::var("RUST_LOG") {
        builder.parse_filters(&var);
    } else {
        // if no RUST_LOG provided, default to logging at the Warn level
        builder.filter(None, LevelFilter::Warn);
    }

    builder.init();
}

fn main() {
    init_logger();


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

            let settings = Settings::build(config_file).unwrap();
            println!("{} {} : {}",
                     Paint::blue("Miner Serve ").bold(),
                     Local::now().format("%Y-%m-%d %H:%M:%S"),
                     Paint::green("Serve start").bold());
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
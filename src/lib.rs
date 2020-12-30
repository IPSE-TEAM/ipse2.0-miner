#![feature(proc_macro_hygiene, decl_macro)]
#![feature(try_trait)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate shells;
#[macro_use]
extern crate log;




mod cli;
mod cmd;
mod constants;
mod storage;
mod error;
mod settings;
mod chain;
mod util;
mod color;
mod crypto;
mod pkcs8;
mod account;










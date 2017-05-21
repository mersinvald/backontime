#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;
extern crate notify;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate lazy_static;

mod global;
mod config;
mod backup_entity;
mod backup;

fn main() {
    let config = config::Config::load("backontime.toml")
        .map_err(|err| {
            println!("error: {}", err);
            std::process::exit(1);
        })
        .unwrap();
    
    let backuper = backup::Backuper::new(config.backups);
    backuper.start();
}


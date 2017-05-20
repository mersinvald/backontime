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

mod config;
mod backup_entity;

fn main() {
    let c = config::Config::load("backontime.toml")
        .map_err(|err| {
            println!("error: {}", err);
            std::process::exit(1);
        });

    println!("{:?}", c);
}


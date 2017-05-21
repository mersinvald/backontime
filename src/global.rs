use std::sync::Mutex;
use std::env;
use log::{LogRecord, LogLevel};
use env_logger::LogBuilder;

lazy_static! {
    pub static ref GLOBAL: Mutex<Global> = Mutex::new(Global::default());
}

pub struct Global {
    pub log_level: LogLevel,
}

impl Default for Global {
    fn default() -> Self {
        Global {
            log_level: LogLevel::Info,
        }
    }
}


impl Global {
    pub fn exec<F: Fn()>(&self, closure: F) {
        closure();
    }

    pub fn setup_logger(&mut self, level: LogLevel) {
        self.log_level = level;

        let format = |record: &LogRecord| format!("{}: {}\t\t\t", record.level(), record.args());

        let mut builder = LogBuilder::new();
        builder.format(format).filter(Some("backontime"), level.to_log_level_filter());

        if let Ok(value) = env::var("RUST_LOG") {
            builder.parse(&value);
        }

        builder.init().unwrap();
    }
}
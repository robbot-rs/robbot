use crate::config::Config;
use chrono::Local;
use log::{Level, Log, Metadata, Record};

pub fn init(config: &Config) {
    log::set_logger(&Logger).unwrap();
    log::set_max_level(config.loglevel);
}

pub struct Logger;

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.target().starts_with("robbot_bin")
    }

    fn log(&self, record: &Record) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S");

        if self.enabled(record.metadata()) {
            println!(
                "[{}] [{}] {}",
                now,
                match record.level() {
                    Level::Error => "ERROR",
                    Level::Warn => "WARN",
                    Level::Info => "INFO",
                    Level::Debug => "DEBUG",
                    Level::Trace => "TRACE",
                },
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

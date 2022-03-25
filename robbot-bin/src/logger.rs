use chrono::Local;
use log::{Level, LevelFilter, Log, Metadata, Record};
use robbot_core::config::Config;

pub fn set_log_level(config: &Config) {
    log::set_max_level(config.loglevel);
}

pub fn init() {
    log::set_logger(&Logger).unwrap();
    log::set_max_level(LevelFilter::Error);
}

pub struct Logger;

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.target().starts_with("robbot")
    }

    fn log(&self, record: &Record) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S");

        if self.enabled(record.metadata()) {
            println!(
                "[{}] [{}:{}] [{}] {}",
                now,
                record.file().unwrap_or("???"),
                record.line().unwrap_or(0),
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

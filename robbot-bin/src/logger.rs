use chrono::Local;
use log::{Level, LevelFilter, Log, Metadata, Record};

pub fn init() {
    log::set_logger(&Logger).unwrap();

    #[cfg(debug_assertions)]
    log::set_max_level(LevelFilter::max());

    #[cfg(not(debug_assertions))]
    log::set_max_level(Level::Info);
}

pub struct Logger;

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.target().starts_with("robbot_bin") && metadata.level() <= Level::Info
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

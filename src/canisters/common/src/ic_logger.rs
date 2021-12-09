use ic_cdk::api;
use log::{LevelFilter, Metadata, Record};

pub struct ICLogger;

impl log::Log for ICLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            api::print(format!("{} - {}", record.level(), record.args()));
        }
    }

    fn flush(&self) {}
}

impl ICLogger {
    pub fn init() {
        match log::set_logger(&ICLogger) {
            Ok(_) => {
                log::set_max_level(LevelFilter::Trace);
            }
            Err(_) => {}
        };
    }
}

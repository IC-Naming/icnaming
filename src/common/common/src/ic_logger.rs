use crate::constants::NAMING_CANISTER_ENV;
use crate::constants::{
    get_log_level_env, is_dev_env, NamingLogLevelEnv, NAMING_CANISTER_LOG_LEVEL_ENV,
};
use ic_cdk::api;
use log::{info, Level, LevelFilter, Metadata, Record};
use std::panic;
use yansi::Paint;

use crate::named_canister_ids::{update_current_canister_name, NAMED_CANISTER_IDS};

pub struct ICLogger;

impl log::Log for ICLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let level = record.level();
            let message = NAMED_CANISTER_IDS.with(|n| {
                let n = n.borrow();
                let name = n.current_name.as_str();
                format!(
                    "{}, {}: {} - {}",
                    name,
                    record.target(),
                    level,
                    record.args()
                )
            });

            let str = match level {
                Level::Error => Paint::red(message),
                Level::Warn => Paint::yellow(message),
                Level::Info => Paint::blue(message),
                Level::Debug => Paint::green(message),
                Level::Trace => Paint::magenta(message),
            };
            api::print(str.to_string());
        }
    }

    fn flush(&self) {}
}

impl ICLogger {
    pub fn init(current_name: &str) {
        update_current_canister_name(current_name);

        if let Some(level_filter) = get_log_level_env() {
            log::set_max_level(level_filter);
        } else if is_dev_env() {
            log::set_max_level(LevelFilter::Trace);
        } else {
            log::set_max_level(LevelFilter::Off);
        }
        match get_log_level_env() {
            Some(level_filter) => log::set_max_level(level_filter),
            None => {
                if is_dev_env() {
                    log::set_max_level(LevelFilter::Trace);
                } else {
                    log::set_max_level(LevelFilter::Off);
                }
            }
        }

        if is_dev_env() && log::set_logger(&ICLogger).is_ok() {
            panic::set_hook(Box::new(|data| {
                let message = format!("{}", data);
                api::print(Paint::red(message).to_string());
            }));
            info!("current wasm is a {} package", NAMING_CANISTER_ENV);
        }
    }
}

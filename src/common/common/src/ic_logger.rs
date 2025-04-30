use crate::constants::NAMING_CANISTER_ENV;
use crate::constants::{get_log_level_env, is_dev_env};
use ic_cdk::api;
use log::{info, Level, Metadata, Record};
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
                Level::Error => message.red(),
                Level::Warn => message.yellow(),
                Level::Info => message.blue(),
                Level::Debug => message.green(),
                Level::Trace => message.magenta(),
            };
            api::debug_print(str.to_string());
        }
    }

    fn flush(&self) {}
}

impl ICLogger {
    pub fn init(current_name: &str) {
        update_current_canister_name(current_name);
        log::set_max_level(get_log_level_env());

        if is_dev_env() && log::set_logger(&ICLogger).is_ok() {
            panic::set_hook(Box::new(|data| {
                let message = format!("{}", data);
                api::debug_print(message.red().to_string());
            }));
            info!("current wasm is a {} package", NAMING_CANISTER_ENV);
        }
    }
}

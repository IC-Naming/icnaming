use std::sync::Once;

use common::ic_logger::ICLogger;

static INIT: Once = Once::new();

pub(crate) fn initialize() {
    INIT.call_once(canister_module_init);
}

fn canister_module_init() {
    ICLogger::init();
}

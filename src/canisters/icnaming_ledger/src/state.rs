use std::sync::Once;

use candid::encode_args;
use ic_cdk::{api, storage};
use ic_cdk_macros::*;
use log::info;

use common::ic_logger::ICLogger;
use common::named_canister_ids::{ensure_current_canister_id_match, get_named_get_canister_id};
use common::state::StableState;

thread_local! {
    pub static STATE : State = State::default();
}

#[derive(Default)]
pub struct State {
    // NOTE: When adding new persistent fields here, ensure that these fields
// are being persisted in the `replace` method below.
}

impl State {
    pub fn replace(&self, new_state: State) {}
}

impl StableState for State {
    fn encode(&self) -> Vec<u8> {
        vec![]
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        Ok(State {})
    }
}

static INIT: Once = Once::new();

pub(crate) fn canister_module_init() {
    INIT.call_once(|| {
        ICLogger::init();
    });
    ensure_current_canister_id_match("icnaming_ledger");
}

#[init]
fn init_function() {
    canister_module_init();
}

#[pre_upgrade]
fn pre_upgrade() {
    // STATE.with(|s| {
    //     let bytes = s.encode();
    //     match storage::stable_save((&bytes,)) {
    //         Ok(_) => {
    //             info!("Saved state before upgrade");
    //             ()
    //         }
    //         Err(e) => api::trap(format!("Failed to save state before upgrade: {:?}", e).as_str()),
    //     };
    // });
}

#[post_upgrade]
fn post_upgrade() {
    // STATE.with(|s| match storage::stable_restore::<(Vec<u8>,)>() {
    //     Ok(bytes) => {
    //         let new_state = State::decode(bytes.0).expect("Decoding stable memory failed");
    //
    //         s.replace(new_state);
    //         canister_module_init();
    //         info!("Loaded state after upgrade");
    //     }
    //     Err(e) => api::trap(format!("Failed to restored state after upgrade: {:?}", e).as_str()),
    // });
}

use std::borrow::Borrow;
use std::cell::RefCell;
use std::sync::Once;

use candid::{candid_method, decode_args, encode_args};
use ic_cdk::{api, storage};
use ic_cdk_macros::*;
use log::info;

use common::ic_logger::ICLogger;
use common::named_canister_ids::{ensure_current_canister_id_match, CanisterNames};
use common::state::StableState;

use crate::resolver_store::ResolverStore;

thread_local! {
    pub static STATE : State = State::default();
}

#[derive(Default)]
pub struct State {
    // NOTE: When adding new persistent fields here, ensure that these fields
    // are being persisted in the `replace` method below.
    pub(crate) resolver_store: RefCell<ResolverStore>,
}

impl State {
    pub fn replace(&self, new_state: State) {
        self.resolver_store.replace(new_state.resolver_store.take());
    }
}

impl StableState for State {
    fn encode(&self) -> Vec<u8> {
        encode_args((self.resolver_store.borrow().encode(),)).unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let (resolver_store_bytes,) = decode_args(&bytes).unwrap();

        Ok(State {
            resolver_store: RefCell::new(ResolverStore::decode(resolver_store_bytes)?),
        })
    }
}

static INIT: Once = Once::new();

fn guard_func() -> Result<(), String> {
    INIT.call_once(|| {
        ICLogger::init("resolver");
    });
    ensure_current_canister_id_match(CanisterNames::Resolver)
}

#[init(guard = "guard_func")]
#[candid_method(init)]
fn init_function() {
    info!("init function called");
}

#[pre_upgrade(guard = "guard_func")]
fn pre_upgrade() {
    STATE.with(|s| {
        let bytes = s.encode();
        match storage::stable_save((&bytes,)) {
            Ok(_) => {
                info!("Saved state before upgrade");
                ()
            }
            Err(e) => api::trap(format!("Failed to save state before upgrade: {:?}", e).as_str()),
        };
    });
}

#[post_upgrade(guard = "guard_func")]
fn post_upgrade() {
    STATE.with(|s| match storage::stable_restore::<(Vec<u8>,)>() {
        Ok(bytes) => {
            let new_state = State::decode(bytes.0).expect("Decoding stable memory failed");

            s.replace(new_state);
            info!("Loaded state after upgrade");
        }
        Err(e) => api::trap(format!("Failed to restored state after upgrade: {:?}", e).as_str()),
    });
}

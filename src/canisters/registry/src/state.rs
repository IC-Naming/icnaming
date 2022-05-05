use std::borrow::Borrow;
use std::cell::RefCell;
use std::sync::Once;

use candid::{candid_method, decode_args, encode_args};
use ic_cdk::{api, storage};
use ic_cdk_macros::*;
use log::info;

use common::ic_logger::ICLogger;
use common::named_canister_ids::{
    ensure_current_canister_id_match, get_named_get_canister_id, CanisterNames,
};
use common::state::StableState;

use crate::registry_store::RegistryStore;
use crate::service::RegistriesService;

thread_local! {
    pub static STATE : State = State::default();
}

#[derive(Default)]
pub struct State {
    // NOTE: When adding new persistent fields here, ensure that these fields
    // are being persisted in the `replace` method below.
    pub(crate) registry_store: RefCell<RegistryStore>,
}

impl State {
    pub fn replace(&self, new_state: State) {
        self.registry_store.replace(new_state.registry_store.take());
    }
}

impl StableState for State {
    fn encode(&self) -> Vec<u8> {
        encode_args((self.registry_store.borrow().encode(),)).unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let (registry_store_bytes,) = decode_args(&bytes).unwrap();

        Ok(State {
            registry_store: RefCell::new(RegistryStore::decode(registry_store_bytes)?),
        })
    }
}

static INIT: Once = Once::new();

fn guard_func() -> Result<(), String> {
    INIT.call_once(|| {
        ICLogger::init("registry");
    });
    ensure_current_canister_id_match(CanisterNames::Registry)
}

#[init(guard = "guard_func")]
#[candid_method(init)]
fn init_function() {
    // insert top level name
    let registrar = get_named_get_canister_id(CanisterNames::Registrar);
    let mut service = RegistriesService::new();
    service.set_top_icp_name(registrar).unwrap();

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

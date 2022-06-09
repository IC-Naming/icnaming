use candid::{CandidType, Deserialize};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Once;

use candid::{candid_method, decode_args, encode_args, Principal};
use ic_cdk::{api, storage};
use ic_cdk_macros::*;
use log::info;

use common::ic_logger::ICLogger;
use common::named_canister_ids::{
    ensure_current_canister_id_match, update_dev_named_canister_ids, CanisterNames,
};
use common::state::StableState;

use crate::user_favorite_store::UserFavoriteStore;

thread_local! {
    pub static STATE : State = State::default();
}

#[derive(Default)]
pub struct State {
    // NOTE: When adding new persistent fields here, ensure that these fields
    // are being persisted in the `replace` method below.
    pub(crate) user_favorite_store: RefCell<UserFavoriteStore>,
}

impl State {
    pub fn replace(&self, new_state: State) {
        self.user_favorite_store
            .replace(new_state.user_favorite_store.take());
    }
}

impl StableState for State {
    fn encode(&self) -> Vec<u8> {
        encode_args((self.user_favorite_store.borrow().encode(),)).unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let (user_favorite_store_bytes,) = decode_args(&bytes).unwrap();

        Ok(State {
            user_favorite_store: RefCell::new(UserFavoriteStore::decode(
                user_favorite_store_bytes,
            )?),
        })
    }
}

static INIT: Once = Once::new();

fn guard_func() -> Result<(), String> {
    INIT.call_once(|| {
        ICLogger::init("favorites");
    });
    ensure_current_canister_id_match(CanisterNames::Favorites)
}

#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    dev_named_canister_ids: HashMap<CanisterNames, Principal>,
}

#[init]
#[candid_method(init)]
#[cfg(feature = "dev_env")]
fn init_function(args: Option<InitArgs>) {
    info!("init function called");
    if let Some(args) = args {
        update_dev_named_canister_ids(&args.dev_named_canister_ids);
    }

    guard_func().unwrap();
}

#[init]
#[candid_method(init)]
#[cfg(not(feature = "dev_env"))]
fn init_function() {
    info!("init function called");
    guard_func().unwrap();
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

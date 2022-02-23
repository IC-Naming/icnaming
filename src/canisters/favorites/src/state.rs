use std::borrow::Borrow;
use std::cell::RefCell;
use std::sync::Once;

use candid::{decode_args, encode_args};
use common::ic_logger::ICLogger;
use common::named_canister_ids::ensure_current_canister_id_match;
use common::named_canister_ids::CANISTER_NAME_FAVORITES;
use ic_cdk::{api, storage};
use ic_cdk_macros::*;
use log::info;

use crate::user_favorite_store::UserFavoriteStore;
use common::state::StableState;

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

pub(crate) fn canister_module_init() {
    INIT.call_once(|| {
        ICLogger::init();
    });
    ensure_current_canister_id_match(CANISTER_NAME_FAVORITES);
}

#[init]
fn init_function() {
    canister_module_init();
}

#[pre_upgrade]
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

#[post_upgrade]
fn post_upgrade() {
    STATE.with(|s| match storage::stable_restore::<(Vec<u8>,)>() {
        Ok(bytes) => {
            let new_state = State::decode(bytes.0).expect("Decoding stable memory failed");

            s.replace(new_state);
            canister_module_init();
            info!("Loaded state after upgrade");
        }
        Err(e) => api::trap(format!("Failed to restored state after upgrade: {:?}", e).as_str()),
    });
}

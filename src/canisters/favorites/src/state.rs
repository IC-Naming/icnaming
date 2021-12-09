use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;

use candid::{CandidType, Deserialize, Principal};
use ic_cdk::storage;
use ic_cdk_macros::*;
use ic_kit::ic;
use log::info;

use crate::models::*;
use crate::startup::initialize;

thread_local! {
    pub(crate) static USER_SET: RefCell<UserFavoriteSet> = RefCell::new(UserFavoriteSet::new());
}

#[init]
fn init_function() {
    initialize();
}

#[derive(Debug, Clone, CandidType, Deserialize)]
pub(crate) struct UpgradePayloadStable {
    pub user_favorites: UserFavoriteSetStable,
}

#[derive(Debug, Clone, CandidType, Deserialize)]
pub(crate) struct UserFavoriteSetStable {
    pub(crate) user_favorites: HashMap<Principal, UserFavoriteStable>,
}

#[derive(Debug, Clone, CandidType, Deserialize)]
pub(crate) struct UserFavoriteStable {
    pub favorites: Vec<String>,
    pub last_update_time: u64,
}

#[pre_upgrade]
fn pre_upgrade() {
    match storage::stable_save((UpgradePayloadStable {
        user_favorites: USER_SET.with(|user_set| {
            let set = user_set.borrow();
            UserFavoriteSetStable::from(set.deref())
        }),
    },))
    {
        Ok(_) => {
            info!("Saved state before upgrade");
            ()
        }
        Err(e) => ic::trap(format!("Failed to save state before upgrade: {:?}", e).as_str()),
    };
}

#[post_upgrade]
fn post_upgrade() {
    match storage::stable_restore::<(UpgradePayloadStable,)>() {
        Ok(payload) => {
            initialize();
            info!("Start to restored state after upgrade");
            let payload = payload.0;

            let user_favorites = payload.user_favorites;
            USER_SET.with(|user_set| {
                let mut user_set = user_set.borrow_mut();
                *user_set = UserFavoriteSet::from(&user_favorites);
            });

            info!("Loaded state after upgrade");
        }
        Err(e) => ic::trap(format!("Failed to restored state after upgrade: {:?}", e).as_str()),
    }
}

use std::cell::RefCell;
use std::collections::HashMap;

use candid::{CandidType, Deserialize};
use ic_cdk::{api, storage};
use ic_cdk_macros::*;
use log::info;

use common::state::{
    get_stable_named_principal, load_stable_named_principal, NamedPrincipalStable,
};

use crate::models::*;
use crate::startup::initialize;
use crate::state::models::RegistryStable;

mod models;

thread_local! {
    pub static REGISTRIES: RefCell<HashMap<String, Registry>> = RefCell::new(HashMap::new());
}

#[derive(Debug, Clone, CandidType, Deserialize)]
struct UpgradePayloadStable {
    named_principals: NamedPrincipalStable,
    registries: HashMap<String, RegistryStable>,
}

#[init]
fn init_function() {
    initialize();
}

#[pre_upgrade]
fn pre_upgrade() {
    match storage::stable_save((UpgradePayloadStable {
        named_principals: get_stable_named_principal(),
        registries: REGISTRIES.with(|registries| {
            let registries = registries.borrow();
            let mut registries_stable = HashMap::new();
            for (name, registry) in registries.iter() {
                registries_stable.insert(name.clone(), RegistryStable::from(registry));
            }
            registries_stable
        }),
    },))
    {
        Ok(_) => {
            info!("Saved state before upgrade");
            ()
        }
        Err(e) => api::trap(format!("Failed to save state before upgrade: {:?}", e).as_str()),
    }
}

#[post_upgrade]
fn post_upgrade() {
    match storage::stable_restore::<(UpgradePayloadStable,)>() {
        Ok(payload) => {
            initialize();

            info!("Start to restored state after upgrade");
            let payload = payload.0;

            let principal = payload.named_principals;
            load_stable_named_principal(&principal);

            let registries = payload.registries;
            REGISTRIES.with(|state| {
                let mut state = state.borrow_mut();
                state.clear();
                for (name, registry) in registries.iter() {
                    state.insert(name.clone(), Registry::from(registry));
                }
            });
            info!("End of restored state after upgrade");
        }
        Err(e) => api::trap(format!("Failed to restored state after upgrade: {:?}", e).as_str()),
    }
}

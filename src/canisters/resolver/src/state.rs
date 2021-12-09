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
use crate::state::models::ResolverStable;

mod models;
thread_local! {
    pub static RESOLVERS: RefCell<HashMap<String, Resolver>> = RefCell::new(HashMap::new());

}

#[derive(Debug, Clone, CandidType, Deserialize)]
struct UpgradePayloadStable {
    named_principals: NamedPrincipalStable,
    resolvers: HashMap<String, ResolverStable>,
}

#[init]
fn init_function() {
    initialize();
}

#[pre_upgrade]
fn pre_upgrade() {
    match storage::stable_save((UpgradePayloadStable {
        named_principals: get_stable_named_principal(),
        resolvers: RESOLVERS.with(|resolvers| {
            let resolvers = resolvers.borrow();
            let map = resolvers
                .iter()
                .map(|(name, resolver)| (name.clone(), ResolverStable::from(resolver)))
                .collect();
            map
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

            let resolvers = payload.resolvers;
            RESOLVERS.with(|state| {
                let mut state = state.borrow_mut();
                state.clear();
                for (name, resolver) in resolvers.iter() {
                    state.insert(name.clone(), Resolver::from(resolver));
                }
            });

            info!("End of restored state after upgrade");
        }
        Err(e) => api::trap(format!("Failed to restored state after upgrade: {:?}", e).as_str()),
    }
}

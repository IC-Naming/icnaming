use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;

use candid::{CandidType, Deserialize};
use ic_cdk::{api, storage};
use ic_cdk_macros::*;
use log::info;

use common::state::{
    get_stable_named_principal, load_stable_named_principal, NamedPrincipalStable,
};

use crate::models::*;
use crate::settings::Settings;
use crate::startup::initialize;
use crate::state::models::{RegistrationStable, SettingsStable};

mod models;

thread_local! {
    pub static SETTINGS : RefCell<Settings> = RefCell::new(Settings::new());
    pub static REGISTRATIONS: RefCell<HashMap<String, Registration>> = RefCell::new(HashMap::new());
}

#[derive(Debug, Clone, CandidType, Deserialize)]
struct UpgradePayloadStable {
    named_principals: NamedPrincipalStable,
    registrations: HashMap<String, RegistrationStable>,
    settings: SettingsStable,
}

#[init]
fn init_function() {
    initialize();
}

#[pre_upgrade]
fn pre_upgrade() {
    match storage::stable_save((UpgradePayloadStable {
        named_principals: get_stable_named_principal(),
        registrations: REGISTRATIONS.with(|r| {
            let mut re = HashMap::new();
            for (k, v) in r.borrow().iter() {
                re.insert(k.clone(), RegistrationStable::from(v));
            }
            re
        }),
        settings: SETTINGS.with(|s| {
            let settings = s.borrow();
            SettingsStable::from(settings.deref())
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

            let registrations_stable = payload.registrations;
            REGISTRATIONS.with(|state| {
                let mut map = state.borrow_mut();
                map.clear();
                for (k, v) in registrations_stable.iter() {
                    map.insert(k.clone(), Registration::from(v));
                }
            });

            let settings_stable = payload.settings;
            SETTINGS.with(|state| {
                let mut settings = state.borrow_mut();
                *settings = Settings::from(&settings_stable);
            });
            info!("Loaded state after upgrade");
        }
        Err(e) => api::trap(format!("Failed to restored state after upgrade: {:?}", e).as_str()),
    }
}

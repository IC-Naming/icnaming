use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::str::FromStr;

use crate::constants::*;
use candid::Principal;
use log::{debug, info};

thread_local! {
    pub static NAME_DPRINCIPALS :RefCell<NamedPrincipals> = RefCell::new(NamedPrincipals::new());
}

pub struct NamedPrincipals {
    pub principals: HashMap<&'static str, HashSet<Principal>>,
}

impl Display for NamedPrincipals {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (name, principals) in self.principals.iter() {
            write!(f, "{}:\n", name)?;
            for principal in principals.iter() {
                write!(f, "  {}\n", principal)?;
            }
        }
        Ok(())
    }
}

impl NamedPrincipals {
    pub fn new() -> NamedPrincipals {
        let mut map = HashMap::new();
        map.insert(
            PRINCIPAL_NAME_ADMIN,
            lines_hashset(NAMING_PRINCIPAL_NAME_ADMIN),
        );
        map.insert(
            PRINCIPAL_NAME_STATE_EXPORTER,
            lines_hashset(NAMING_PRINCIPAL_NAME_STATE_EXPORTER),
        );
        map.insert(
            PRINCIPAL_NAME_TIMER_TRIGGER,
            lines_hashset(NAMING_PRINCIPAL_NAME_TIMER_TRIGGER),
        );

        let result = NamedPrincipals { principals: map };
        info!("named principals: {}", &result);
        result
    }

    pub fn set(&mut self, name: &'static str, principals: HashSet<Principal>) {
        self.principals.insert(name, principals);
    }
}

pub(crate) fn lines_hashset(s: &str) -> HashSet<Principal> {
    let mut set = HashSet::new();
    for line in s.split("||||") {
        if line.starts_with("#") {
            continue;
        }
        if line.is_empty() {
            continue;
        }
        let principal = Principal::from_str(line).unwrap();
        set.insert(principal);
    }
    set
}

pub fn is_named_principal(name: &str, principal: &Principal) -> bool {
    let result =
        NAME_DPRINCIPALS.with(|store| {
            let store = store.borrow();
            store.principals.get(name).unwrap().contains(principal)
        });
    if is_dev_env() {
        debug!("is_named_principal({}, {}) = {}", name, principal, result);
        if !result {
            NAME_DPRINCIPALS.with(|store| {
                let store = store.borrow();
                store.principals.get(name).unwrap().iter().for_each(|p| {
                    debug!("  {}", p);
                });
            });
        }
    }
    result
}

pub fn get_named_principals(name: &str) -> HashSet<Principal> {
    NAME_DPRINCIPALS.with(|store| {
        let store = store.borrow();
        store.principals.get(name).unwrap().clone()
    })
}

pub const PRINCIPAL_NAME_ADMIN: &str = "user:administrator";
pub const PRINCIPAL_NAME_STATE_EXPORTER: &str = "app:state_exporter";
pub const PRINCIPAL_NAME_TIMER_TRIGGER: &str = "app:timer_trigger";
pub const PRINCIPAL_DICP_RECEIVER: &str = "wallet:dicp_receiver";

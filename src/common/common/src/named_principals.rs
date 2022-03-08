use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::str::FromStr;

use candid::Principal;
use log::{debug, info};

thread_local! {
    pub static NAME_DPRINCIPALS :NamedPrincipals = NamedPrincipals::new();
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
        let (administrator, state_exporter, timer_trigger) = {
            #[cfg(feature = "dev_canister")]
            {
                (
                    include_str!("../../../configs/dev/principal_registrar_admin.in"),
                    include_str!("../../../configs/dev/principal_state_exporter.in"),
                    include_str!("../../../configs/dev/principal_timer_trigger.in"),
                )
            }
            #[cfg(feature = "staging_canister")]
            {
                (
                    include_str!("../../../configs/staging/principal_registrar_admin.in"),
                    include_str!("../../../configs/staging/principal_state_exporter.in"),
                    include_str!("../../../configs/staging/principal_timer_trigger.in"),
                )
            }
            #[cfg(feature = "production_canister")]
            {
                (
                    include_str!("../../../configs/production/principal_registrar_admin.in"),
                    include_str!("../../../configs/production/principal_state_exporter.in"),
                    include_str!("../../../configs/production/principal_timer_trigger.in"),
                )
            }
        };
        let mut map = HashMap::new();
        map.insert(PRINCIPAL_NAME_ADMIN, lines_hashset(administrator));
        map.insert(PRINCIPAL_NAME_STATE_EXPORTER, lines_hashset(state_exporter));
        map.insert(PRINCIPAL_NAME_TIMER_TRIGGER, lines_hashset(timer_trigger));

        let result = NamedPrincipals { principals: map };
        info!("named principals: {}", &result);
        result
    }
}

fn lines_hashset(s: &str) -> HashSet<Principal> {
    let mut set = HashSet::new();
    for line in s.lines() {
        if line.starts_with("#") {
            continue;
        }
        let principal = Principal::from_str(line).unwrap();
        set.insert(principal);
    }
    set
}

pub fn is_named_principal(name: &str, principal: &Principal) -> bool {
    let result =
        NAME_DPRINCIPALS.with(|store| store.principals.get(name).unwrap().contains(principal));
    #[cfg(feature = "dev_canister")]
    {
        debug!("is_named_principal({}, {}) = {}", name, principal, result);
        if !result {
            NAME_DPRINCIPALS.with(|store| {
                store.principals.get(name).unwrap().iter().for_each(|p| {
                    debug!("  {}", p);
                });
            });
        }
    }
    result
}

pub fn get_named_principals(name: &str) -> HashSet<Principal> {
    NAME_DPRINCIPALS.with(|store| store.principals.get(name).unwrap().clone())
}

pub const PRINCIPAL_NAME_ADMIN: &str = "user:administrator";
pub const PRINCIPAL_NAME_STATE_EXPORTER: &str = "app:state_exporter";
pub const PRINCIPAL_NAME_TIMER_TRIGGER: &str = "app:timer_trigger";

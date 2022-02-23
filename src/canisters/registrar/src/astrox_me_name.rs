use std::collections::HashSet;

use candid::{CandidType, Deserialize, Principal};

/// Name must be imported before import_limit_time.  2022-04-01 00:00:00 UTC
pub const ASTROX_ME_NAME_IMPORT_LIMIT_TIME: u64 = 1648771200 * 1000000000;

/// Astrox Me had launch a event with ICNaming to let people register their name.
/// Since this event is launched before the main net online, we need to import the name after the main net is online.
/// This code will be removed after all the name is imported.
#[derive(Clone, Debug)]
pub struct AstroXMeName {
    names: HashSet<String>,
    owner_canister_id: Principal,
}

impl AstroXMeName {
    pub fn get_names(&self) -> &HashSet<String> {
        &self.names
    }

    pub fn get_owner_canister_id(&self) -> &Principal {
        &self.owner_canister_id
    }
}

impl Default for AstroXMeName {
    fn default() -> Self {
        let names = {
            #[cfg(feature = "dev_canister")]
            {
                include_str!("../../../configs/dev/names_astrox_me_event.in")
            }
            #[cfg(feature = "staging_canister")]
            {
                include_str!("../../../configs/staging/names_astrox_me_event.in")
            }
            #[cfg(feature = "production_canister")]
            {
                include_str!("../../../configs/production/names_astrox_me_event.in")
            }
        };
        let names = names.lines().map(|s| s.to_string()).collect();
        let canister_id_str = {
            #[cfg(feature = "dev_canister")]
            {
                include_str!("../../../configs/dev/canister_ids_astrox_me.in")
            }
            #[cfg(feature = "staging_canister")]
            {
                include_str!("../../../configs/staging/canister_ids_astrox_me.in")
            }
            #[cfg(feature = "production_canister")]
            {
                include_str!("../../../configs/production/canister_ids_astrox_me.in")
            }
        };
        let owner_canister_id = Principal::from_text(canister_id_str).unwrap();
        Self {
            names,
            owner_canister_id,
        }
    }
}

#[derive(CandidType, Deserialize)]
pub struct ImportedStats {
    pub total: u32,
    pub imported: u32,
    pub not_imported: u32,
}

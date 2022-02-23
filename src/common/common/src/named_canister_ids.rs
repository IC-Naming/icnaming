use std::collections::HashMap;
use std::str::FromStr;

use candid::Principal;
use ic_cdk::api;

thread_local! {
    pub static NAMED_CANISTER_IDS :NamedCanisterIds = NamedCanisterIds::new();
}

pub struct NamedCanisterIds {
    pub canisters: HashMap<&'static str, Principal>,
}

impl NamedCanisterIds {
    pub fn new() -> NamedCanisterIds {
        let (registrar, registry, resolver, icnaming_ledger, cycles_minting, favorites, ledger) = {
            #[cfg(feature = "dev_canister")]
            {
                (
                    include_str!("../../../configs/dev/canister_ids_registrar.in"),
                    include_str!("../../../configs/dev/canister_ids_registry.in"),
                    include_str!("../../../configs/dev/canister_ids_resolver.in"),
                    include_str!("../../../configs/dev/canister_ids_icnaming_ledger.in"),
                    include_str!("../../../configs/dev/canister_ids_cycles_minting.in"),
                    include_str!("../../../configs/dev/canister_ids_favorites.in"),
                    include_str!("../../../configs/dev/canister_ids_ledger.in"),
                )
            }
            #[cfg(feature = "staging_canister")]
            {
                (
                    include_str!("../../../configs/staging/canister_ids_registrar.in"),
                    include_str!("../../../configs/staging/canister_ids_registry.in"),
                    include_str!("../../../configs/staging/canister_ids_resolver.in"),
                    include_str!("../../../configs/staging/canister_ids_icnaming_ledger.in"),
                    include_str!("../../../configs/staging/canister_ids_cycles_minting.in"),
                    include_str!("../../../configs/staging/canister_ids_favorites.in"),
                    include_str!("../../../configs/staging/canister_ids_ledger.in"),
                )
            }
            #[cfg(feature = "production_canister")]
            {
                (
                    include_str!("../../../configs/production/canister_ids_registrar.in"),
                    include_str!("../../../configs/production/canister_ids_registry.in"),
                    include_str!("../../../configs/production/canister_ids_resolver.in"),
                    include_str!("../../../configs/production/canister_ids_icnaming_ledger.in"),
                    include_str!("../../../configs/production/canister_ids_cycles_minting.in"),
                    include_str!("../../../configs/production/canister_ids_favorites.in"),
                    include_str!("../../../configs/production/canister_ids_ledger.in"),
                )
            }
        };
        let mut map = HashMap::new();
        map.insert(
            CANISTER_NAME_REGISTRAR,
            Principal::from_str(registrar).unwrap(),
        );
        map.insert(
            CANISTER_NAME_REGISTRY,
            Principal::from_str(registry).unwrap(),
        );
        map.insert(
            CANISTER_NAME_RESOLVER,
            Principal::from_str(resolver).unwrap(),
        );
        map.insert(
            CANISTER_NAME_ICNAMING_LEDGER,
            Principal::from_str(icnaming_ledger).unwrap(),
        );
        map.insert(
            CANISTER_NAME_CYCLES_MINTING,
            Principal::from_str(cycles_minting).unwrap(),
        );
        map.insert(
            CANISTER_NAME_FAVORITES,
            Principal::from_str(favorites).unwrap(),
        );
        map.insert(CANISTER_NAME_LEDGER, Principal::from_str(ledger).unwrap());

        NamedCanisterIds { canisters: map }
    }

    pub fn get_canister_id(&self, name: &str) -> Principal {
        let id = self.canisters.get(name);
        if id.is_none() {
            panic!("Canister {} not found", name);
        }
        id.unwrap().clone()
    }
}

pub fn get_named_get_canister_id(name: &str) -> Principal {
    NAMED_CANISTER_IDS.with(|n| n.get_canister_id(name))
}

pub fn ensure_current_canister_id_match(name: &str) {
    let current = api::id();
    let expected = get_named_get_canister_id(name);
    if current != expected {
        panic!("Current canister id does not match expected canister id. Expected: {:?}, Current: {:?}", expected, current);
    }
}

pub const CANISTER_NAME_REGISTRY: &str = "registry";
pub const CANISTER_NAME_REGISTRAR: &str = "registrar";
pub const CANISTER_NAME_RESOLVER: &str = "resolver";
pub const CANISTER_NAME_FAVORITES: &str = "favorites";
pub const CANISTER_NAME_ICNAMING_LEDGER: &str = "icnaming_ledger";
pub const CANISTER_NAME_CYCLES_MINTING: &str = "cycles_minting";
pub const CANISTER_NAME_LEDGER: &str = "ledger";

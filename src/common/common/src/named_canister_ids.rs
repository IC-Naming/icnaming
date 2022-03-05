use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use candid::Principal;
use ic_cdk::api;
use log::info;

thread_local! {
    pub static NAMED_CANISTER_IDS :NamedCanisterIds = NamedCanisterIds::new();
}

pub struct NamedCanisterIds {
    pub canisters: HashMap<&'static str, Principal>,
}

impl Display for NamedCanisterIds {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (name, canister) in self.canisters.iter() {
            write!(f, "{} = {}\n", name, canister)?;
        }
        Ok(())
    }
}

impl NamedCanisterIds {
    pub fn new() -> NamedCanisterIds {
        let (
            registrar,
            registrar_control_gateway,
            registry,
            resolver,
            icnaming_ledger,
            cycles_minting,
            favorites,
            ledger,
        ) = {
            #[cfg(feature = "dev_canister")]
            {
                (
                    include_str!("../../../configs/dev/canister_ids_registrar.in"),
                    include_str!("../../../configs/dev/canister_ids_registrar_control_gateway.in"),
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
                    include_str!(
                        "../../../configs/staging/canister_ids_registrar_control_gateway.in"
                    ),
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
                    include_str!(
                        "../../../configs/production/canister_ids_registrar_control_gateway.in"
                    ),
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
            CANISTER_NAME_REGISTRAR_CONTROL_GATEWAY,
            Principal::from_str(registrar_control_gateway).unwrap(),
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

        let result = NamedCanisterIds { canisters: map };
        info!("named canister ids: {}", &result);
        result
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

pub fn is_named_canister_id(name: &str, id: &Principal) -> bool {
    NAMED_CANISTER_IDS.with(|n| n.canisters.get(name).map(|x| x == id).unwrap_or(false))
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
pub const CANISTER_NAME_REGISTRAR_CONTROL_GATEWAY: &str = "registrar_control_gateway";
pub const CANISTER_NAME_RESOLVER: &str = "resolver";
pub const CANISTER_NAME_FAVORITES: &str = "favorites";
pub const CANISTER_NAME_ICNAMING_LEDGER: &str = "icnaming_ledger";
pub const CANISTER_NAME_CYCLES_MINTING: &str = "cycles_minting";
pub const CANISTER_NAME_LEDGER: &str = "ledger";

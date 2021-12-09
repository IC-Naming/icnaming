use std::collections::HashSet;

use candid::{CandidType, Deserialize, Principal};

use common::dto::IRegistryUsers;

use crate::models::Registry;

#[derive(Debug, Clone, CandidType, Deserialize)]
pub(crate) struct RegistryStable {
    name: String,
    owner: Principal,
    ttl: u64,
    resolver: Principal,
    operators: HashSet<Principal>,
}

// Registry -> RegistryStable
impl From<&Registry> for RegistryStable {
    fn from(registry: &Registry) -> Self {
        let operators = match registry.get_operators() {
            Some(operators) => operators.iter().cloned().collect(),
            None => HashSet::new(),
        };
        RegistryStable {
            name: registry.get_name().to_string(),
            owner: registry.get_owner().clone(),
            ttl: registry.get_ttl(),
            resolver: registry.get_resolver(),
            operators,
        }
    }
}

// RegistryStable -> Registry
impl From<&RegistryStable> for Registry {
    fn from(registry: &RegistryStable) -> Self {
        let mut re = Registry::new(
            registry.name.clone(),
            registry.owner.clone(),
            registry.ttl,
            registry.resolver,
        );
        re.set_operators(registry.operators.clone());
        re
    }
}

use std::collections::HashSet;

use candid::Principal;

use common::constants::DEFAULT_TTL;
use common::dto::{IRegistryUsers, RegistryDto, RegistryUsers};

#[derive(Debug, Clone)]
pub struct Registry {
    name: String,
    owner: Principal,
    ttl: u64,
    resolver: Principal,
    operators: HashSet<Principal>,
}

impl IRegistryUsers for Registry {
    fn get_operators(&self) -> Option<&HashSet<Principal>> {
        Some(&self.operators)
    }

    fn get_owner(&self) -> &Principal {
        &self.owner
    }
}

// Registry  -> RegistryDto
impl From<&Registry> for RegistryDto {
    fn from(registry: &Registry) -> Self {
        RegistryDto {
            name: registry.name.clone(),
            owner: registry.owner.clone(),
            ttl: registry.ttl,
            resolver: registry.resolver.clone(),
        }
    }
}

impl Default for Registry {
    fn default() -> Self {
        Registry {
            name: "".to_string(),
            owner: Principal::anonymous(),
            ttl: DEFAULT_TTL,
            resolver: Principal::anonymous(),
            operators: HashSet::new(),
        }
    }
}

impl Registry {
    pub fn new(name: String, owner: Principal, ttl: u64, resolver: Principal) -> Self {
        Registry {
            name,
            owner,
            ttl,
            resolver,
            operators: HashSet::new(),
        }
    }
    pub(crate) fn set_operators(&mut self, operators: HashSet<Principal>) {
        self.operators = operators;
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn get_owner(&self) -> &Principal {
        &self.owner
    }
    pub fn set_owner(&mut self, owner: Principal) {
        self.owner = owner;
    }
    pub fn set_ttl(&mut self, ttl: u64) {
        self.ttl = ttl;
    }
    pub fn get_ttl(&self) -> u64 {
        self.ttl
    }
    pub fn set_resolver(&mut self, resolver: Principal) {
        self.resolver = resolver;
    }
    pub fn get_resolver(&self) -> Principal {
        self.resolver.clone()
    }
    pub fn add_operator(&mut self, operator: &Principal) {
        self.operators.insert(operator.clone());
    }
    pub fn remove_operator(&mut self, operator: &Principal) {
        self.operators.remove(operator);
    }

    pub(crate) fn get_users(&self) -> RegistryUsers {
        RegistryUsers {
            owner: self.owner.clone(),
            operators: self.operators.clone(),
        }
    }
}

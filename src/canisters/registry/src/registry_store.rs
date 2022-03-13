use std::collections::{HashMap, HashSet};

use candid::{decode_args, encode_args, CandidType, Deserialize, Principal};

use common::constants::DEFAULT_TTL;
use common::dto::{IRegistryUsers, RegistryDto, RegistryUsers};
use common::state::StableState;

#[derive(CandidType, Deserialize, Debug, Clone)]
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
    pub fn get_operator_count(&self) -> usize {
        self.operators.len()
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

#[derive(Default)]
pub struct RegistryStore {
    registries: HashMap<String, Registry>,
}

impl StableState for RegistryStore {
    fn encode(&self) -> Vec<u8> {
        encode_args((&self.registries,)).unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let (registries,): (HashMap<String, Registry>,) = decode_args(&bytes).unwrap();

        Ok(RegistryStore { registries })
    }
}

impl RegistryStore {
    pub fn new() -> Self {
        RegistryStore {
            registries: HashMap::new(),
        }
    }

    pub fn get_registries(&self) -> &HashMap<String, Registry> {
        &self.registries
    }

    pub fn get_registries_mut(&mut self) -> &mut HashMap<String, Registry> {
        &mut self.registries
    }

    pub fn get_sub_names(&self, parent_name: &str) -> Vec<String> {
        let end_parts = format!(".{}", parent_name);
        let mut sub_names = vec![];
        for (name, _) in self.registries.iter() {
            if name.ends_with(end_parts.as_str()) {
                sub_names.push(name.to_string());
            }
        }
        sub_names
    }

    pub fn remove_names(&mut self, names: &Vec<String>) {
        for name in names {
            self.registries.remove(name);
        }
    }

    pub fn add_registry(&mut self, registry: Registry) {
        self.registries.insert(registry.name.clone(), registry);
    }

    pub fn get_registry(&self, name: &str) -> Option<&Registry> {
        self.registries.get(name)
    }

    pub fn update_owner(&mut self, name: &str, owner: Principal) {
        if let Some(registry) = self.registries.get_mut(name) {
            registry.set_owner(owner);
        }
    }
}

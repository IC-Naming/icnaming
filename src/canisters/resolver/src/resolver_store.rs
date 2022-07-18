use std::collections::HashMap;

use candid::{decode_args, encode_args, CandidType, Deserialize};

use common::state::StableState;

#[derive(CandidType, Deserialize, Clone)]
pub struct Resolver {
    name: String,
    string_value_map: HashMap<String, String>,
}

impl Resolver {
    pub fn new(name: String) -> Resolver {
        Resolver {
            name,
            string_value_map: HashMap::new(),
        }
    }
    pub(crate) fn get_name(&self) -> &String {
        &self.name
    }
    pub(crate) fn set_string_map(&mut self, map: &HashMap<String, String>) {
        self.string_value_map = map.clone();
    }
    pub fn set_record_value(&mut self, key: String, value: String) {
        self.string_value_map.insert(key, value);
    }
    pub fn remove_record_value(&mut self, key: String) {
        self.string_value_map.remove(&key);
    }

    pub(crate) fn get_record_value(&self) -> &HashMap<String, String> {
        &self.string_value_map
    }
    pub fn contains_key(&self, key: &str) -> bool {
        self.string_value_map.contains_key(key)
    }
    pub fn string_value_map(&self) -> &HashMap<String, String> {
        &self.string_value_map
    }
}

#[derive(Default)]
pub struct ResolverStore {
    resolvers: HashMap<String, Resolver>,
}

impl StableState for ResolverStore {
    fn encode(&self) -> Vec<u8> {
        encode_args((&self.resolvers,)).unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let (resolvers,): (HashMap<String, Resolver>,) = decode_args(&bytes).unwrap();

        Ok(ResolverStore { resolvers })
    }
}

impl ResolverStore {
    pub fn new() -> ResolverStore {
        ResolverStore {
            resolvers: HashMap::new(),
        }
    }

    pub fn get_resolvers(&self) -> &HashMap<String, Resolver> {
        &self.resolvers
    }

    pub fn get_resolvers_mut(&mut self) -> &mut HashMap<String, Resolver> {
        &mut self.resolvers
    }
    pub fn ensure_created(&mut self, name: &str) {
        if !self.resolvers.contains_key(name) {
            self.resolvers
                .insert(name.to_string(), Resolver::new(name.to_string()));
        }
    }
    pub fn clean_up_names(&mut self, names: &Vec<String>) {
        for name in names {
            self.resolvers.remove(name);
        }
    }
}

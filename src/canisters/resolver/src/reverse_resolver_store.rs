use candid::{decode_args, encode_args, Principal};
use common::state::StableState;
use std::collections::HashMap;

#[derive(Default)]
pub struct ReverseResolverStore {
    primary_names: HashMap<Principal, String>,
    primary_names_reverse: HashMap<String, Principal>,
}

impl StableState for ReverseResolverStore {
    fn encode(&self) -> Vec<u8> {
        encode_args((&self.primary_names, &self.primary_names_reverse)).unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let (primary_names, primary_names_reverse): (
            HashMap<Principal, String>,
            HashMap<String, Principal>,
        ) = decode_args(&bytes).unwrap();

        Ok(ReverseResolverStore {
            primary_names,
            primary_names_reverse,
        })
    }
}

impl ReverseResolverStore {
    pub fn set_primary_name(&mut self, principal: Principal, name: String) {
        if let Some(old_name) = self.primary_names.insert(principal, name.clone()) {
            self.primary_names_reverse.remove(&old_name);
        }
        self.primary_names_reverse.insert(name, principal);
    }
    pub fn remove_primary_name(&mut self, principal: Principal) -> Option<String> {
        if let Some(name) = self.primary_names.remove(&principal) {
            self.primary_names_reverse.remove(&name);
            Some(name)
        } else {
            None
        }
    }
    pub fn remove_primary_name_by_name(&mut self, name: &String) -> Option<Principal> {
        if let Some(principal) = self.primary_names_reverse.remove(name) {
            self.primary_names.remove(&principal);
            Some(principal)
        } else {
            None
        }
    }
    pub fn get_primary_name(&self, principal: &Principal) -> Option<&String> {
        self.primary_names.get(principal)
    }
    pub fn get_primary_name_reverse(&self, name: &String) -> Option<&Principal> {
        self.primary_names_reverse.get(name)
    }
}

use candid::{decode_args, encode_args, CandidType, Deserialize, Principal};
use common::state::StableState;
use common::token_identifier::TokenIndex;
use std::borrow::Borrow;
use std::collections::HashMap;

#[derive(Deserialize, CandidType, Clone, Hash, Eq, PartialEq, Debug)]
pub struct RegistrationName(pub String);

#[derive(Default)]
pub struct TokenIndexStore {
    index: TokenIndex,
    registrations: HashMap<TokenIndex, RegistrationName>,
}

impl TokenIndexStore {
    pub fn new() -> TokenIndexStore {
        TokenIndexStore {
            index: TokenIndex(0),
            registrations: HashMap::new(),
        }
    }

    pub fn import_from_registration_store(&mut self, names: Vec<String>) -> usize {
        let mut count = 0;
        for name in names {
            if self
                .try_add_registration_name(RegistrationName(name))
                .is_some()
            {
                count += 1;
            }
        }
        count
    }

    pub fn try_add_registration_name(&mut self, name: RegistrationName) -> Option<TokenIndex> {
        if self.registrations.values().any(|val| val == &name) {
            return None;
        }
        let next_index = self.next_index();
        self.registrations.insert(next_index, name);
        Some(next_index)
    }

    fn next_index(&mut self) -> TokenIndex {
        self.index.0 += 1;
        self.index.clone()
    }
    pub fn get_index(&self) -> TokenIndex {
        self.index.clone()
    }
    pub fn get_registrations(&self) -> &HashMap<TokenIndex, RegistrationName> {
        &self.registrations
    }
    pub fn get_registration(&self, index: &TokenIndex) -> Option<RegistrationName> {
        self.registrations.get(index).cloned()
    }
}

impl StableState for TokenIndexStore {
    fn encode(&self) -> Vec<u8> {
        encode_args((&self.index, &self.registrations)).unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let (index, registrations): (TokenIndex, HashMap<TokenIndex, RegistrationName>) =
            decode_args(&bytes).unwrap();

        Ok(TokenIndexStore {
            index,
            registrations,
        })
    }
}

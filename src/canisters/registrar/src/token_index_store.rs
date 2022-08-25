use crate::state::NEXT_TOKEN_INDEX;
use candid::{decode_args, encode_args, CandidType, Deserialize};
use common::state::StableState;
use common::token_identifier::TokenIndex;
use log::error;
use std::cell::RefCell;
use std::collections::{BinaryHeap, HashMap};
use std::hash::Hash;
use std::ops::Deref;
use std::rc::Rc;
use std::vec::Vec;

pub fn next_token_index() -> TokenIndex {
    NEXT_TOKEN_INDEX.with(|state| {
        let mut state = state.borrow_mut();
        let id = *state;
        let new_id = TokenIndex(id.get_value() + 1);
        *state = new_id;
        new_id
    })
}

pub fn get_current_token_index() -> TokenIndex {
    NEXT_TOKEN_INDEX.with(|state| {
        let state = state.borrow();
        *state
    })
}

#[derive(Deserialize, CandidType, Clone, Hash, Eq, PartialEq, Debug, Ord, PartialOrd)]
pub struct RegistrationName {
    id: TokenIndex,
    name: String,
}

impl RegistrationName {
    pub fn new(name: String) -> RegistrationName {
        RegistrationName {
            id: next_token_index(),
            name,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_id(&self) -> TokenIndex {
        self.id
    }

    pub fn get_metadata(&self) -> Option<Vec<u8>> {
        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), self.get_name());
        match encode_args((metadata,)) {
            Ok(data) => Some(data),
            Err(e) => {
                error!("error encoding metadata: {:?}", e);
                None
            }
        }
    }
}

pub type RegistrationNameRef = Rc<RefCell<RegistrationName>>;
pub type RegistrationNames = BinaryHeap<RegistrationNameRef>;

#[derive(Default)]
pub struct TokenIndexStore {
    registrations: RegistrationNames,
    token_indexes: HashMap<TokenIndex, RegistrationNameRef>,
    name_indexes: HashMap<String, RegistrationNameRef>,
}

impl TokenIndexStore {
    pub fn new() -> Self {
        TokenIndexStore::default()
    }

    pub fn import_from_registration_store(&mut self, names: &[String]) -> usize {
        let mut count = 0;
        for name in names {
            if self.try_add_registration_name(name).is_some() {
                count += 1;
            }
        }
        count
    }

    pub fn try_add_registration_name(&mut self, name: &String) -> Option<TokenIndex> {
        if self.get_registration_by_name(name).is_some() {
            return None;
        }
        let registration_name = RegistrationName::new(name.to_owned());
        let token_id = registration_name.id.clone();
        let registration_name_ref = Rc::new(RefCell::new(registration_name));
        self.token_indexes
            .insert(token_id, registration_name_ref.clone());
        self.name_indexes
            .insert(name.to_owned(), registration_name_ref.clone());
        self.registrations.push(registration_name_ref);
        Some(token_id)
    }

    pub fn get_registrations(&self) -> Vec<RegistrationName> {
        self.registrations
            .iter()
            .map(|registration_name_ref| registration_name_ref.deref().deref().borrow().clone())
            .collect()
    }
    pub fn get_registration(&self, index: &TokenIndex) -> Option<&RegistrationNameRef> {
        self.token_indexes.get(index)
    }
    pub fn get_registration_by_name(&self, name: &String) -> Option<&RegistrationNameRef> {
        self.name_indexes.get(name)
    }
}

impl StableState for TokenIndexStore {
    fn encode(&self) -> Vec<u8> {
        encode_args((&self.registrations, &self.token_indexes, &self.name_indexes)).unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let (registrations, token_indexes, name_indexes): (
            RegistrationNames,
            HashMap<TokenIndex, RegistrationNameRef>,
            HashMap<String, RegistrationNameRef>,
        ) = decode_args(&bytes).unwrap();

        Ok(TokenIndexStore {
            registrations,
            token_indexes,
            name_indexes,
        })
    }
}

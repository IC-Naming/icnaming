use candid::{decode_args, encode_args, CandidType, Deserialize};
use common::errors::NamingError;

use common::state::StableState;
use common::token_identifier::TokenIndex;
use log::error;
use std::cell::RefCell;
use std::collections::{BinaryHeap, HashMap};
use std::hash::Hash;
use std::ops::Deref;
use std::rc::Rc;
use std::vec::Vec;

#[derive(Clone, Hash, Eq, PartialEq, Debug, Ord, PartialOrd)]
pub struct RegistrationName {
    index: TokenIndex,
    name: String,
}

impl RegistrationName {
    pub fn new(index: TokenIndex, name: String) -> RegistrationName {
        RegistrationName { index, name }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_index(&self) -> TokenIndex {
        self.index
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
    index: TokenIndex,
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
            match self.try_add_registration_name(name) {
                Ok(_) => count += 1,
                Err(e) => error!("import registration name error: {:?}", e),
            }
        }
        count
    }

    pub fn try_add_registration_name(&mut self, name: &String) -> Result<TokenIndex, NamingError> {
        if self.get_registration_by_name(name).is_some() {
            return Err(NamingError::RegistrationNameIsAlreadyIndexed {
                name: name.to_owned(),
            });
        }
        let token_id = self.next_token_index();
        let registration_name = RegistrationName::new(token_id, name.to_owned());
        self.add_registration_name(&registration_name);
        Ok(token_id)
    }
    pub fn add_registration_name(&mut self, registration_name: &RegistrationName) -> TokenIndex {
        let token_id = registration_name.index.to_owned();
        let name = registration_name.name.to_owned();
        let registration_name_ref = Rc::new(RefCell::new(registration_name.to_owned()));
        self.token_indexes
            .insert(token_id, registration_name_ref.to_owned());
        self.name_indexes
            .insert(name, registration_name_ref.to_owned());
        self.registrations.push(registration_name_ref);
        token_id
    }

    pub fn get_registrations(&self) -> &RegistrationNames {
        &self.registrations
    }
    pub fn get_registration(&self, index: &TokenIndex) -> Option<&RegistrationNameRef> {
        self.token_indexes.get(index)
    }
    pub fn get_registration_by_name(&self, name: &String) -> Option<&RegistrationNameRef> {
        self.name_indexes.get(name)
    }
    fn next_token_index(&mut self) -> TokenIndex {
        let new_index = TokenIndex(self.index.get_value() + 1);
        self.index = new_index.clone();
        new_index
    }
    pub fn get_current_token_index(&self) -> TokenIndex {
        self.index.clone()
    }
}

#[derive(Clone, CandidType, Deserialize)]
struct StableRegistrationName {
    index: TokenIndex,
    name: String,
}

impl From<&RegistrationName> for StableRegistrationName {
    fn from(registration_name: &RegistrationName) -> Self {
        StableRegistrationName {
            index: registration_name.index.clone(),
            name: registration_name.name.clone(),
        }
    }
}

impl From<&StableRegistrationName> for RegistrationName {
    fn from(stable_registration_name: &StableRegistrationName) -> Self {
        RegistrationName {
            index: stable_registration_name.index.clone(),
            name: stable_registration_name.name.clone(),
        }
    }
}

impl StableState for TokenIndexStore {
    fn encode(&self) -> Vec<u8> {
        let stable_registrations: Vec<StableRegistrationName> = self
            .token_indexes
            .values()
            .map(|registration| {
                let registration = registration.borrow();
                StableRegistrationName::from(registration.deref())
            })
            .collect();
        let token_index = self.index;
        encode_args((token_index, stable_registrations)).unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let (token_index, stable_registrations): (TokenIndex, Vec<StableRegistrationName>) =
            decode_args(&bytes).unwrap();

        let mut token_index_store = TokenIndexStore::default();
        for registration in stable_registrations {
            token_index_store.add_registration_name(&RegistrationName::from(&registration));
        }
        token_index_store.index = token_index;
        Ok(token_index_store)
    }
}

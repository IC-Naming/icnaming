use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use candid::{decode_args, encode_args, CandidType, Deserialize, Principal};
use common::naming::{FirstLevelName, NormalizedName};

use common::state::StableState;

/// Name registration
#[derive(CandidType, Deserialize, Eq, PartialEq, Clone)]
pub struct Registration {
    /// The owner of the name
    owner: Principal,
    /// Domain name
    name: String,
    /// When the name is expired
    expired_at: u64,
    /// When the name is registered
    created_at: u64,
}

impl Registration {
    pub fn new(owner: Principal, name: String, expired_at: u64, created_at: u64) -> Registration {
        Registration {
            owner,
            name,
            expired_at,
            created_at,
        }
    }

    pub fn is_owner(&self, principal: &Principal) -> bool {
        self.owner == *principal
    }
    pub fn set_owner(&mut self, owner: Principal) {
        self.owner = owner;
    }
    pub(crate) fn get_owner(&self) -> Principal {
        self.owner.clone()
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_expired_at(&self) -> u64 {
        self.expired_at
    }
    pub fn get_created_at(&self) -> u64 {
        self.created_at
    }
}

impl Debug for Registration {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Registration {{ owner: {}, name: {}, expired_at: {}, created_at: {} }}",
            self.owner, self.name, self.expired_at, self.created_at
        )
    }
}

#[derive(Default)]
pub struct RegistrationStore {
    pub registrations: HashMap<String, Registration>,
}

impl RegistrationStore {
    pub fn get_registrations(&self) -> &HashMap<String, Registration> {
        &self.registrations
    }

    pub fn get_registration(&self, name: &FirstLevelName) -> Option<&Registration> {
        self.registrations.get(name.0.get_name())
    }

    pub fn add_registration(&mut self, registration: Registration) {
        self.registrations
            .insert(registration.name.clone(), registration);
    }
    pub fn transfer_registration(&mut self, name: String, owner: Principal) {
        self.registrations.entry(name).and_modify(|registration| {
            registration.set_owner(owner);
        });
    }

    pub fn has_registration(&self, name: &FirstLevelName) -> bool {
        self.registrations.contains_key(name.0.get_name())
    }

    pub fn update_expired_at(&mut self, name: &FirstLevelName, expired_at: u64) {
        self.registrations
            .entry(name.0.get_name().clone())
            .and_modify(|registration| {
                registration.expired_at = expired_at;
            });
    }

    pub fn get_user_own_registration_count(&self, user: &Principal) -> usize {
        self.registrations
            .values()
            .filter(|registration| registration.is_owner(user))
            .count()
    }
}

impl StableState for RegistrationStore {
    fn encode(&self) -> Vec<u8> {
        encode_args((&self.registrations,)).unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let (registrations,): (HashMap<String, Registration>,) = decode_args(&bytes).unwrap();

        Ok(RegistrationStore { registrations })
    }
}

/// Details of a registration
#[derive(Debug, Deserialize, CandidType, Eq, PartialEq)]
pub struct RegistrationDetails {
    /// The owner of the registration
    owner: Principal,
    /// Domain name
    name: String,
    /// When the registration expires, ms since epoch
    expired_at: u64,
    /// When the registration was created, ms since epoch
    created_at: u64,
}

/// Details of a registration
#[derive(CandidType)]
pub struct RegistrationDto {
    /// Domain name
    name: String,
    /// When the registration expires, ms since epoch
    expired_at: u64,
    /// When the registration was created, ms since epoch
    created_at: u64,
}

impl From<&Registration> for RegistrationDetails {
    fn from(registration: &Registration) -> RegistrationDetails {
        RegistrationDetails {
            owner: registration.owner.to_owned(),
            name: registration.name.to_owned(),
            expired_at: registration.expired_at / 1_000_000,
            created_at: registration.created_at / 1_000_000,
        }
    }
}

// Registration -> RegistrationDto
impl From<&Registration> for RegistrationDto {
    fn from(registration: &Registration) -> RegistrationDto {
        RegistrationDto {
            name: registration.name.clone(),
            expired_at: registration.expired_at / 1_000_000,
            created_at: registration.created_at / 1_000_000,
        }
    }
}

use candid::{CandidType, Principal};

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Registration {
    owner: Principal,
    name: String,
    expired_at: u64,
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

#[derive(CandidType)]
pub struct RegistrationDetails {
    owner: Principal,
    name: String,
    expired_at: u64,
    created_at: u64,
}

#[derive(CandidType)]
pub(crate) struct RegistrationDto {
    name: String,
    expired_at: u64,
    created_at: u64,
}

impl From<&Registration> for RegistrationDetails {
    fn from(registration: &Registration) -> RegistrationDetails {
        RegistrationDetails {
            owner: registration.owner.to_owned(),
            name: registration.name.to_owned(),
            expired_at: registration.expired_at,
            created_at: registration.created_at,
        }
    }
}

// Registration -> RegistrationDto
impl From<&Registration> for RegistrationDto {
    fn from(registration: &Registration) -> RegistrationDto {
        RegistrationDto {
            name: registration.name.clone(),
            expired_at: registration.expired_at,
            created_at: registration.created_at,
        }
    }
}

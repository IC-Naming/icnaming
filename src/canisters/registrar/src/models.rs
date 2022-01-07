use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use candid::{CandidType, Deserialize, Principal};
use log::info;

#[cfg(test)]
mod tests;

/// Name registration
#[derive(Eq, PartialEq, Debug, Clone)]
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

/// Details of a registration
#[derive(CandidType)]
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
pub(crate) struct RegistrationDto {
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

/// Quota type to be used for registration
#[derive(Deserialize, CandidType, Clone, Hash, Eq, PartialEq, Debug)]
pub enum QuotaType {
    /// The length of name's the first part in chars must be equal to the value.
    /// e.g. LenEq(3) means that the first part of the name must be 3 chars long.
    LenEq(u8),
    /// The length of name's the first part in chars must be more than or equal to the value.
    /// e.g. LenGt(3) means that the first part of the name must be at least 3 chars long.
    LenGte(u8),
}

impl Display for QuotaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            QuotaType::LenEq(len) => write!(f, "len_eq({})", len),
            QuotaType::LenGte(len) => write!(f, "len_gte({})", len),
        }
    }
}

pub struct UserQuotaManager {
    user_quotas: HashMap<Principal, HashMap<QuotaType, u32>>,
}

impl UserQuotaManager {
    pub fn new() -> UserQuotaManager {
        UserQuotaManager {
            user_quotas: HashMap::new(),
        }
    }

    pub fn get_quota(&self, principal: &Principal, quota_type: &QuotaType) -> Option<u32> {
        self.user_quotas
            .get(principal)
            .and_then(|quotas| quotas.get(quota_type).cloned())
    }

    pub fn add_quota(&mut self, principal: Principal, quota_type: QuotaType, quota: u32) {
        let quotas = self
            .user_quotas
            .entry(principal.clone())
            .or_insert(HashMap::new());
        // increment the quota
        let old_value = quotas.entry(quota_type.clone()).or_insert(0);
        *old_value += quota;
        info!("updated quotas {} {} {}", principal, quota_type, *old_value);
    }

    pub fn sub_quota(&mut self, principal: &Principal, quota_type: &QuotaType, diff: u32) -> bool {
        let quotas = self
            .user_quotas
            .entry(principal.clone())
            .or_insert(HashMap::new());
        let quota_value = quotas.get(quota_type).cloned().unwrap_or(0);
        if quota_value >= diff {
            let new_value = quota_value - diff;
            if new_value == 0 {
                quotas.remove(quota_type);
            } else {
                quotas.insert(quota_type.clone(), new_value);
            }
            info!("updated quotas {} {} {}", principal, quota_type, new_value);
            true
        } else {
            false
        }
    }

    pub fn get_quotas(&self) -> Vec<(Principal, QuotaType, u32)> {
        let mut result = Vec::new();
        for (principal, quotas) in &self.user_quotas {
            for (quota_type, quota) in quotas {
                result.push((principal.clone(), quota_type.clone(), *quota));
            }
        }
        result
    }

    pub fn load_quotas(&mut self, quotas: Vec<(Principal, QuotaType, u32)>) {
        for (principal, quota_type, quota) in quotas {
            self.add_quota(principal, quota_type, quota);
        }
    }
}

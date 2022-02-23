use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use candid::{decode_args, encode_args, CandidType, Deserialize, Principal};
use log::info;

use common::state::StableState;

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

#[derive(Default)]
pub struct UserQuotaStore {
    user_quotas: HashMap<Principal, HashMap<QuotaType, u32>>,
}

impl UserQuotaStore {
    pub fn new() -> UserQuotaStore {
        UserQuotaStore {
            user_quotas: HashMap::new(),
        }
    }

    pub fn get_quota(&self, principal: &Principal, quota_type: &QuotaType) -> Option<u32> {
        self.user_quotas
            .get(principal)
            .and_then(|quotas| quotas.get(quota_type).cloned())
    }

    pub fn add_quota(&mut self, principal: Principal, quota_type: QuotaType, diff: u32) {
        assert!(diff > 0);
        let quotas = self
            .user_quotas
            .entry(principal.clone())
            .or_insert(HashMap::new());
        // increment the quota
        let old_value = quotas.entry(quota_type.clone()).or_insert(0);
        *old_value += diff;
        info!("updated quotas {} {} {}", principal, quota_type, *old_value);
    }

    pub fn sub_quota(&mut self, principal: &Principal, quota_type: &QuotaType, diff: u32) -> bool {
        assert!(diff > 0);
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
}

impl StableState for UserQuotaStore {
    fn encode(&self) -> Vec<u8> {
        encode_args((&self.user_quotas,)).unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        #[allow(clippy::type_complexity)]
        let (user_quotas,): (HashMap<Principal, HashMap<QuotaType, u32>>,) =
            decode_args(&bytes).unwrap();

        Ok(UserQuotaStore { user_quotas })
    }
}

#[cfg(test)]
mod tests;

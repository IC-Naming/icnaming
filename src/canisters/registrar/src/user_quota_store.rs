use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use candid::{decode_args, encode_args, CandidType, Deserialize, Principal};
use log::info;

use common::state::StableState;

/// Quota type to be used for registration
#[derive(Deserialize, Copy, CandidType, Clone, Hash, Eq, PartialEq, Debug)]
pub enum QuotaType {
    /// The length of name's the first part in chars must be equal to the value.
    /// e.g. LenEq(3) means that the first part of the name must be 3 chars long.
    LenEq(u8),
    /// The length of name's the first part in chars must be more than or equal to the value.
    /// e.g. LenGt(3) means that the first part of the name must be at least 3 chars long.
    LenGte(u8),
}

impl FromStr for QuotaType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // find () in s
        let mut iter = s.splitn(2, '(');
        let name = iter.next().unwrap();
        let args = iter.next().unwrap();
        let args = args.trim_end_matches(')');
        match name {
            "LenEq" => Ok(QuotaType::LenEq(u8::from_str(args).unwrap())),
            "LenGte" => Ok(QuotaType::LenGte(u8::from_str(args).unwrap())),
            _ => Err(()),
        }
    }
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

impl StableState for UserQuotaStore {
    fn encode(&self) -> Vec<u8> {
        encode_args((&self.user_quotas,)).unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let (user_quotas,): (HashMap<Principal, HashMap<QuotaType, u32>>,) =
            decode_args(&bytes).unwrap();

        Ok(UserQuotaStore { user_quotas })
    }
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

    pub fn get_user_quotas(&self) -> &HashMap<Principal, HashMap<QuotaType, u32>> {
        &self.user_quotas
    }

    pub fn transfer_quota(
        &mut self,
        from: &Principal,
        to: &Principal,
        quota_type: &QuotaType,
        diff: u32,
    ) -> bool {
        assert!(diff > 0);
        let from_quotas = self.user_quotas.get_mut(from);
        if from_quotas.is_none() {
            return false;
        }
        let from_quotas = from_quotas.unwrap();
        let quota_value = from_quotas.get(quota_type).cloned().unwrap_or(0);
        if quota_value < diff {
            return false;
        }
        let new_value = quota_value - diff;
        if new_value == 0 {
            from_quotas.remove(quota_type);
        } else {
            from_quotas.insert(quota_type.clone(), new_value);
        }
        let to_quotas = self.user_quotas.entry(to.clone()).or_insert(HashMap::new());
        let old_value = to_quotas.entry(quota_type.clone()).or_insert(0);
        *old_value += diff;
        info!(
            "transfer quotas {} {} {} with diff {}",
            from, to, quota_type, diff
        );
        true
    }
}

#[cfg(test)]
mod tests;

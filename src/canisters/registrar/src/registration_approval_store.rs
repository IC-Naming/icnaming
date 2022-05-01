use std::collections::HashMap;

use candid::{decode_args, encode_args, CandidType, Deserialize, Principal};
use common::naming::FirstLevelName;

use common::state::StableState;

#[derive(CandidType, Deserialize)]
struct ApprovalRecord {
    approved_to: Principal,
    approval_at: u64,
}

#[derive(Default)]
pub struct RegistrationApprovalStore {
    approvals: HashMap<String, ApprovalRecord>,
}

impl StableState for RegistrationApprovalStore {
    fn encode(&self) -> Vec<u8> {
        encode_args((&self.approvals,)).unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let (approvals,): (HashMap<String, ApprovalRecord>,) = decode_args(&bytes).unwrap();

        Ok(RegistrationApprovalStore { approvals })
    }
}

impl RegistrationApprovalStore {
    pub fn set_approval(
        &mut self,
        name: &FirstLevelName,
        approved_to: &Principal,
        approval_at: u64,
    ) {
        if approved_to == &Principal::anonymous() {
            self.approvals.remove(name.0.get_name());
        } else {
            self.approvals.insert(
                name.to_string(),
                ApprovalRecord {
                    approved_to: approved_to.clone(),
                    approval_at,
                },
            );
        }
    }

    pub fn remove_approval(&mut self, name: &FirstLevelName) {
        self.approvals.remove(name.0.get_name());
    }

    pub fn is_approved_to(&self, name: &FirstLevelName, approved_to: &Principal) -> bool {
        if let Some(approval) = self.approvals.get(name.0.get_name()) {
            if approval.approved_to == *approved_to {
                return true;
            }
        }
        false
    }

    pub fn has_approved_to(&self, name: &FirstLevelName) -> bool {
        self.approvals.contains_key(name.0.get_name())
    }
}

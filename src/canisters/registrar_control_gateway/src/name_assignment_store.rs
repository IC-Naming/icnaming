use std::collections::HashMap;

use candid::{decode_args, encode_args, CandidType, Deserialize, Principal};

use common::state::StableState;

#[derive(CandidType, Deserialize)]
struct AssignmentRecord {
    owner: Principal,
    assigned_at: u64,
}

#[derive(Default)]
pub struct NameAssignmentStore {
    assignments: HashMap<String, AssignmentRecord>,
}

impl StableState for NameAssignmentStore {
    fn encode(&self) -> Vec<u8> {
        encode_args((&self.assignments,)).unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let (assignments,): (HashMap<String, AssignmentRecord>,) = decode_args(&bytes).unwrap();

        Ok(NameAssignmentStore { assignments })
    }
}

impl NameAssignmentStore {
    pub fn new() -> NameAssignmentStore {
        NameAssignmentStore {
            assignments: HashMap::new(),
        }
    }

    pub fn name_assigned(&self, name: &str) -> bool {
        self.assignments.contains_key(name)
    }

    pub fn add_assignment(&mut self, name: &str, owner: Principal, assigned_at: u64) {
        self.assignments
            .insert(name.to_string(), AssignmentRecord { owner, assigned_at });
    }
}

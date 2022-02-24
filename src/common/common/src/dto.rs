use std::collections::HashSet;

use std::io::Write;

use candid::{CandidType, Deserialize, Principal};
use flate2::write::ZlibEncoder;
use flate2::Compression;

use crate::constants::{
    PAGE_INPUT_MAX_LIMIT, PAGE_INPUT_MAX_OFFSET, PAGE_INPUT_MIN_LIMIT, PAGE_INPUT_MIN_OFFSET,
};
use crate::errors::{ErrorInfo, ICNSError, ICNSResult};

#[cfg(test)]
mod tests;

#[derive(CandidType, Deserialize)]
pub struct GetPageInput {
    pub offset: usize,
    pub limit: usize,
}

impl GetPageInput {
    pub fn validate(&self) -> ICNSResult<()> {
        let max_offset = PAGE_INPUT_MAX_OFFSET;
        let min_offset = PAGE_INPUT_MIN_OFFSET;
        if self.offset > max_offset || self.offset < min_offset {
            return Err(ICNSError::ValueShouldBeInRangeError {
                field: "offset".to_string(),
                min: min_offset,
                max: max_offset,
            });
        }
        let max_limit = PAGE_INPUT_MAX_LIMIT;
        let min_limit = PAGE_INPUT_MIN_LIMIT;
        if self.limit > max_limit || self.limit < min_limit {
            return Err(ICNSError::ValueShouldBeInRangeError {
                field: "limit".to_string(),
                min: min_limit,
                max: max_limit,
            });
        }
        Ok(())
    }
}

#[derive(CandidType, Deserialize)]
pub struct GetPageOutput<T> {
    pub items: Vec<T>,
}

impl<T> GetPageOutput<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self { items }
    }
}

pub trait IRegistryUsers {
    fn get_operators(&self) -> Option<&HashSet<Principal>>;
    fn get_owner(&self) -> &Principal;
    fn can_operate(&self, principal: &Principal) -> bool {
        if self.is_owner(principal) {
            return true;
        }
        if let Some(operators) = self.get_operators() {
            return operators.contains(principal);
        }
        false
    }
    fn is_owner(&self, principal: &Principal) -> bool {
        self.get_owner() == principal
    }
}

#[derive(Debug, CandidType, Deserialize)]
pub struct RegistryUsers {
    pub owner: Principal,
    pub operators: HashSet<Principal>,
}

impl IRegistryUsers for RegistryUsers {
    fn get_operators(&self) -> Option<&HashSet<Principal>> {
        Some(&self.operators)
    }
    fn get_owner(&self) -> &Principal {
        &self.owner
    }
}

#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct RegistryDto {
    pub name: String,
    pub owner: Principal,
    pub ttl: u64,
    pub resolver: Principal,
}

#[derive(CandidType)]
pub struct StateExportData {
    pub state_data: Vec<u8>,
}

#[derive(CandidType)]
pub enum StateExportResponse {
    Ok(StateExportData),
    Err(ErrorInfo),
}

impl StateExportResponse {
    pub fn new(result: ICNSResult<StateExportData>) -> StateExportResponse {
        match result {
            Ok(stats) => StateExportResponse::Ok(stats),
            Err(err) => StateExportResponse::Err(err.into()),
        }
    }
}

pub fn to_state_export_data(source_state_data: Vec<u8>) -> StateExportData {
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(&source_state_data).unwrap();
    let data = e.finish().unwrap();
    StateExportData { state_data: data }
}

use std::collections::HashSet;
use std::fmt::{Display, Formatter};

use std::io::{Read, Write};

use candid::{CandidType, Deserialize, Principal};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;

use crate::constants::{
    PAGE_INPUT_MAX_LIMIT, PAGE_INPUT_MAX_OFFSET, PAGE_INPUT_MIN_LIMIT, PAGE_INPUT_MIN_OFFSET,
};
use crate::errors::{ErrorInfo, NamingError, ServiceResult};

#[cfg(test)]
mod tests;

#[derive(CandidType, Deserialize)]
pub struct GetPageInput {
    pub offset: usize,
    pub limit: usize,
}

impl GetPageInput {
    pub fn validate(&self) -> ServiceResult<()> {
        let max_offset = PAGE_INPUT_MAX_OFFSET;
        let min_offset = PAGE_INPUT_MIN_OFFSET;
        if self.offset > max_offset || self.offset < min_offset {
            return Err(NamingError::ValueShouldBeInRangeError {
                field: "offset".to_string(),
                min: min_offset,
                max: max_offset,
            });
        }
        let max_limit = PAGE_INPUT_MAX_LIMIT;
        let min_limit = PAGE_INPUT_MIN_LIMIT;
        if self.limit > max_limit || self.limit < min_limit {
            return Err(NamingError::ValueShouldBeInRangeError {
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
    pub page_count: u32,
    pub total_count: u32,
}

impl<T> GetPageOutput<T> {
    pub fn new(items: Vec<T>, page_count: u32, total_count: u32) -> Self {
        Self {
            items,
            page_count,
            total_count,
        }
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

#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct ImportQuotaItem {
    pub owner: Principal,
    pub quota_type: String,
    pub diff: u32,
}

#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct ImportQuotaRequest {
    pub items: Vec<ImportQuotaItem>,
    pub hash: Vec<u8>,
}

#[derive(Debug, Clone, CandidType, Deserialize)]
pub enum ImportQuotaStatus {
    Ok,
    AlreadyExists,
}

#[derive(CandidType, Deserialize)]
pub struct LoadStateRequest {
    pub state_data: Vec<u8>,
}

impl Display for LoadStateRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "LoadStateRequest with {} bytes", self.state_data.len())
    }
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
    pub fn new(result: ServiceResult<StateExportData>) -> StateExportResponse {
        match result {
            Ok(stats) => StateExportResponse::Ok(stats),
            Err(err) => StateExportResponse::Err(err.into()),
        }
    }
}

pub fn encode_zlib(data: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).unwrap();
    let data = encoder.finish().unwrap();
    data
}

pub fn decode_zlib(data: &[u8]) -> Vec<u8> {
    let mut d = ZlibDecoder::new(data);
    let mut decoded_data = Vec::new();
    d.read_to_end(&mut decoded_data).unwrap();
    decoded_data
}

pub fn to_state_export_data(source_state_data: Vec<u8>) -> StateExportData {
    StateExportData {
        state_data: encode_zlib(source_state_data.as_slice()),
    }
}

pub fn from_state_export_data(request: LoadStateRequest) -> Vec<u8> {
    decode_zlib(request.state_data.as_slice())
}

#[derive(CandidType)]
pub enum GetStatsResponse<T> {
    Ok(T),
    Err(ErrorInfo),
}

impl<T> GetStatsResponse<T> {
    pub fn new(result: ServiceResult<T>) -> GetStatsResponse<T> {
        match result {
            Ok(stats) => GetStatsResponse::Ok(stats),
            Err(err) => GetStatsResponse::Err(err.into()),
        }
    }
}

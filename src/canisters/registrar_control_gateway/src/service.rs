use std::sync::Arc;

use candid::{CandidType, Deserialize, Principal};

use log::info;

use common::canister_api::ic_impl::RegistrarApi;
use common::canister_api::IRegistrarApi;
use common::dto::{ImportQuotaRequest, ImportQuotaStatus};
use common::errors::ServiceResult;

use common::permissions::must_be_system_owner;

use crate::quota_import_store::ImportError;
use crate::state::STATE;

pub struct GatewayService {
    pub registrar_api: Arc<dyn IRegistrarApi>,
}

#[derive(Debug, Clone, CandidType, Deserialize)]
pub enum ImportQuotaResult {
    Ok,
    AlreadyExists,
    InvalidRequest,
}

#[derive(Debug, Clone, CandidType, Deserialize)]
pub enum AssignNameResult {
    Ok,
    AlreadyAssigned,
    FailFromRegistrar,
}

impl Default for GatewayService {
    fn default() -> Self {
        Self {
            registrar_api: Arc::new(RegistrarApi::default()),
        }
    }
}

impl GatewayService {
    pub async fn import_quota(
        &self,
        caller: &Principal,
        file_content: Vec<u8>,
    ) -> ServiceResult<ImportQuotaResult> {
        must_be_system_owner(caller)?;
        let parse_result = STATE.with(|s| {
            let store = s.quota_import_store.borrow();
            store.verify_and_parse(file_content.as_slice())
        });
        if parse_result.is_err() {
            return match parse_result.err().unwrap() {
                ImportError::FileAlreadyImported => Ok(ImportQuotaResult::AlreadyExists),
                ImportError::FileNotAcceptable => Ok(ImportQuotaResult::InvalidRequest),
            };
        }
        let (items, hashes) = parse_result.unwrap();
        info!("{} items to import", items.len());

        let result = self
            .registrar_api
            .import_quota(ImportQuotaRequest {
                hash: hashes.clone(),
                items,
            })
            .await?;
        let result = match result {
            ImportQuotaStatus::Ok => Ok(()),
            ImportQuotaStatus::AlreadyExists => Err(ImportQuotaResult::AlreadyExists),
        };
        if result.is_err() {
            return Ok(result.err().unwrap());
        }

        // apply items and save hashes
        STATE.with(|s| {
            let mut import_quota_store = s.quota_import_store.borrow_mut();
            info!("file imported, save hashes: {}", hex::encode(&hashes));
            import_quota_store.add_imported_file_hash(hashes);
            Ok(ImportQuotaResult::Ok)
        })
    }

    pub async fn assign_name(
        &self,
        caller: &Principal,
        now: u64,
        name: String,
        owner: Principal,
    ) -> ServiceResult<AssignNameResult> {
        must_be_system_owner(caller)?;

        let name_assigned = STATE.with(|s| {
            let store = s.name_assignment_store.borrow();
            store.name_assigned(name.as_str())
        });
        if name_assigned {
            return Ok(AssignNameResult::AlreadyAssigned);
        }

        let register_result = self
            .registrar_api
            .register_from_gateway(name.clone(), owner)
            .await;

        if register_result.is_err() {
            return Ok(AssignNameResult::FailFromRegistrar);
        }
        let register_result = register_result.unwrap();
        if register_result {
            STATE.with(|s| {
                let mut store = s.name_assignment_store.borrow_mut();
                store.add_assignment(name.as_str(), owner, now);
                Ok(AssignNameResult::Ok)
            })
        } else {
            Ok(AssignNameResult::FailFromRegistrar)
        }
    }
}

#[cfg(test)]
mod tests;

use std::sync::Arc;

use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api;
use log::{error, info};

use common::canister_api::ic_impl::RegistrarApi;
use common::canister_api::IRegistrarApi;
use common::dto::{ImportQuotaRequest, ImportQuotaStatus};
use common::errors::ICNSResult;
use common::metrics_encoder::MetricsEncoder;
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

impl GatewayService {
    pub fn new() -> Self {
        Self {
            registrar_api: Arc::new(RegistrarApi::new()),
        }
    }
    pub(crate) fn get_stats(&self, now: u64) -> Stats {
        let mut stats = Stats::default();
        stats.cycles_balance = api::canister_balance();
        STATE.with(|s| {
            {
                let store = s.name_assignment_store.borrow();
                let assignments = store.get_assignments();
                stats.name_assignments_count = assignments.len() as u64;
            }
            {
                let store = s.quota_import_store.borrow();
                let acceptable_file_hashes = store.get_acceptable_file_hashes();
                stats.acceptable_file_hashes_count = acceptable_file_hashes.len() as u64;

                let imported_file_hashes = store.get_imported_file_hashes();
                stats.imported_file_hashes_count = imported_file_hashes.len() as u64;
            }
        });

        stats
    }

    pub async fn import_quota(
        &self,
        caller: &Principal,
        file_content: Vec<u8>,
    ) -> ICNSResult<ImportQuotaResult> {
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
    ) -> ICNSResult<AssignNameResult> {
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

pub fn encode_metrics(w: &mut MetricsEncoder<Vec<u8>>) -> std::io::Result<()> {
    let service = GatewayService::new();
    let now = api::time();
    let stats = service.get_stats(now);
    w.encode_gauge(
        "icnaming_registrar_control_gateway_cycles_balance",
        stats.cycles_balance as f64,
        "Cycles balance",
    )?;
    w.encode_gauge(
        "icnaming_registrar_control_gateway_acceptable_file_hashes_count",
        stats.acceptable_file_hashes_count as f64,
        "Acceptable file hashes count",
    )?;
    w.encode_gauge(
        "icnaming_registrar_control_gateway_imported_file_hashes_count",
        stats.imported_file_hashes_count as f64,
        "Imported file hashes count",
    )?;
    w.encode_gauge(
        "icnaming_registrar_control_gateway_name_assignments_count",
        stats.name_assignments_count as f64,
        "Assigned names count",
    )?;

    Ok(())
}

#[derive(CandidType, Deserialize, Default)]
pub struct Stats {
    cycles_balance: u64,
    acceptable_file_hashes_count: u64,
    imported_file_hashes_count: u64,
    name_assignments_count: u64,
}

#[cfg(test)]
mod tests;

mod build_gen;
mod http;
mod name_assignment_store;
mod quota_import_store;
mod service;
mod state;

#[path = "../../../common/common_actor/src/actor.rs"]
mod shared_actor;
mod stats_service;

use crate::state::InitArgs;
use common::dto::*;
use common::http::*;
use stats_service::*;
use std::collections::HashMap;

use candid::{candid_method, CandidType, Principal};
use common::errors::BooleanActorResponse;
use common::errors::{ErrorInfo, ServiceResult};
use ic_cdk::api;
use ic_cdk_macros::*;
use log::debug;

use crate::service::{AssignNameResult, GatewayService, ImportQuotaResult};

#[update(name = "import_quota")]
#[candid_method(update, rename = "import_quota")]
pub async fn import_quota(file_content: Vec<u8>) -> ImportQuotaResponse {
    let caller = &api::caller();
    debug!("import_quota: caller: {}", caller);

    let service = GatewayService::default();
    let result = service.import_quota(caller, file_content).await;
    ImportQuotaResponse::new(result)
}

#[derive(CandidType)]
pub enum ImportQuotaResponse {
    Ok(ImportQuotaResult),
    Err(ErrorInfo),
}

impl ImportQuotaResponse {
    pub fn new(result: ServiceResult<ImportQuotaResult>) -> ImportQuotaResponse {
        match result {
            Ok(status) => ImportQuotaResponse::Ok(status),
            Err(err) => ImportQuotaResponse::Err(err.into()),
        }
    }
}

#[update(name = "assign_name")]
#[candid_method(update, rename = "assign_name")]
pub async fn assign_name(name: String, owner: Principal) -> AssignNameResponse {
    let caller = &api::caller();
    debug!("import_quota: caller: {}", caller);
    let now = api::time();

    let service = GatewayService::default();
    let result = service.assign_name(caller, now, name, owner).await;
    AssignNameResponse::new(result)
}

#[derive(CandidType)]
pub enum AssignNameResponse {
    Ok(AssignNameResult),
    Err(ErrorInfo),
}

impl AssignNameResponse {
    pub fn new(result: ServiceResult<AssignNameResult>) -> AssignNameResponse {
        match result {
            Ok(status) => AssignNameResponse::Ok(status),
            Err(err) => AssignNameResponse::Err(err.into()),
        }
    }
}

candid::export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
#[candid_method(query, rename = "__get_candid_interface_tmp_hack")]
fn __export_did_tmp_() -> String {
    __export_service()
}

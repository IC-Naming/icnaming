use std::collections::HashSet;

use candid::{candid_method, CandidType, Principal};
use ic_cdk::api;
use ic_cdk_macros::*;
use log::{debug, error, info};

use common::dto::{
    from_state_export_data, to_state_export_data, GetPageInput, GetPageOutput, LoadStateRequest,
    StateExportResponse,
};
use common::errors::{BooleanActorResponse, ErrorInfo, ICNSError, ICNSResult};
use common::icnaming_ledger_types::BlockHeight;
use common::named_principals::{PRINCIPAL_NAME_STATE_EXPORTER, PRINCIPAL_NAME_TIMER_TRIGGER};
use common::permissions::{must_be_named_principal, must_be_system_owner};
use common::state::StableState;

use crate::service::{AssignNameResult, GatewayService, ImportQuotaResult, Stats};
use crate::state::{State, STATE};

#[query(name = "get_stats")]
#[candid_method(query, rename = "get_stats")]
pub fn get_stats() -> GetStatsActorResponse {
    let now = api::time();
    let service = GatewayService::new();
    let stats = service.get_stats(now);
    GetStatsActorResponse::new(Ok(stats))
}

#[derive(CandidType)]
pub enum GetStatsActorResponse {
    Ok(Stats),
    Err(ErrorInfo),
}

impl GetStatsActorResponse {
    pub fn new(result: ICNSResult<Stats>) -> GetStatsActorResponse {
        match result {
            Ok(stats) => GetStatsActorResponse::Ok(stats),
            Err(err) => GetStatsActorResponse::Err(err.into()),
        }
    }
}

#[update(name = "export_state")]
#[candid_method(update, rename = "export_state")]
pub async fn export_state() -> StateExportResponse {
    let caller = &api::caller();
    let permission_result = must_be_named_principal(caller, PRINCIPAL_NAME_STATE_EXPORTER);
    if permission_result.is_err() {
        return StateExportResponse::new(Err(permission_result.err().unwrap()));
    }

    let source_data = STATE.with(|state| to_state_export_data(state.encode()));
    StateExportResponse::new(Ok(source_data))
}

#[cfg(feature = "dev_canister")]
#[update(name = "load_state")]
#[candid_method(update, rename = "load_state")]
pub async fn load_state(request: LoadStateRequest) -> BooleanActorResponse {
    debug!("load_state: {}", request);
    let caller = &api::caller();
    if must_be_system_owner(caller).is_err() {
        error!("load_state: caller is not system owner");
        return BooleanActorResponse::new(Err(ICNSError::PermissionDenied));
    }
    STATE.with(|s| {
        let bytes = from_state_export_data(request);
        let result = State::decode(bytes);
        if result.is_err() {
            error!("Failed to decode state: {:?}", result.err());
            return BooleanActorResponse::Err(ErrorInfo::from(ICNSError::Unknown));
        }
        let new_state = result.unwrap();
        s.replace(new_state);
        info!("load_state: success");
        return BooleanActorResponse::Ok(true);
    })
}

#[update(name = "import_quota")]
#[candid_method(update, rename = "import_quota")]
pub async fn import_quota(file_content: Vec<u8>) -> ImportQuotaResponse {
    let caller = &api::caller();
    debug!("import_quota: caller: {}", caller);

    let service = GatewayService::new();
    let result = service.import_quota(caller, file_content).await;
    ImportQuotaResponse::new(result)
}

#[derive(CandidType)]
pub enum ImportQuotaResponse {
    Ok(ImportQuotaResult),
    Err(ErrorInfo),
}

impl ImportQuotaResponse {
    pub fn new(result: ICNSResult<ImportQuotaResult>) -> ImportQuotaResponse {
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

    let service = GatewayService::new();
    let result = service.assign_name(caller, now, name, owner).await;
    AssignNameResponse::new(result)
}

#[derive(CandidType)]
pub enum AssignNameResponse {
    Ok(AssignNameResult),
    Err(ErrorInfo),
}

impl AssignNameResponse {
    pub fn new(result: ICNSResult<AssignNameResult>) -> AssignNameResponse {
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

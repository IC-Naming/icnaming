use std::collections::HashMap;

use candid::{candid_method, CandidType};
use ic_cdk::{api, caller};
use ic_cdk_macros::*;

use common::dto::{StateExportResponse, to_state_export_data};
use common::errors::{BooleanActorResponse, ErrorInfo, ICNSError, ICNSResult};
use common::named_canister_ids::CANISTER_NAME_REGISTRY;
use common::named_canister_ids::get_named_get_canister_id;
use common::permissions::must_be_system_owner;
use common::state::StableState;

use crate::service::{ResolverService, Stats};
use crate::state::STATE;

#[query(name = "get_stats")]
#[candid_method(query, rename = "get_stats")]
pub fn get_stats() -> GetStatsResponse {
    let service = ResolverService::new();
    let stats = service.get_stats();
    GetStatsResponse::new(Ok(stats))
}

#[derive(CandidType)]
pub enum GetStatsResponse {
    Ok(Stats),
    Err(ErrorInfo),
}

impl GetStatsResponse {
    pub fn new(result: ICNSResult<Stats>) -> GetStatsResponse {
        match result {
            Ok(data) => GetStatsResponse::Ok(data),
            Err(err) => GetStatsResponse::Err(err.into()),
        }
    }
}

#[update(name = "export_state")]
#[candid_method(update, rename = "export_state")]
pub async fn export_state() -> StateExportResponse {
    let caller = &api::caller();
    let permission_result = must_be_system_owner(caller);
    if permission_result.is_err() {
        return StateExportResponse::new(Err(permission_result.err().unwrap()));
    }

    let source_data = STATE.with(|state| to_state_export_data(state.encode()));
    StateExportResponse::new(Ok(source_data))
}

/// Ensure the resolver is created.
/// Returns true if the resolver is created, false otherwise.
///
/// * `name` - a name. e.g. `hello.icp`
#[update(name = "ensure_resolver_created")]
#[candid_method(update, rename = "ensure_resolver_created")]
fn ensure_resolver_created(name: String) -> BooleanActorResponse {
    let caller = caller();
    let result = if caller != get_named_get_canister_id(CANISTER_NAME_REGISTRY) {
        Err(ICNSError::PermissionDenied)
    } else {
        let mut service = ResolverService::new();
        service.ensure_resolver_created(name.as_str())
    };
    BooleanActorResponse::new(result)
}

/// Set the record values for the name
/// Returns true if the record is set, false otherwise.
///
/// * `name` - a name. e.g. `hello.icp`
/// * `values` - a list of values. e.g. `canister.icp`
#[update(name = "set_record_value")]
#[candid_method(update, rename = "set_record_value")]
async fn set_record_value(
    name: String,
    patch_values: HashMap<String, String>,
) -> BooleanActorResponse {
    let mut service = ResolverService::new();
    let result = service.set_record_value(name.as_str(), patch_values).await;
    BooleanActorResponse::new(result)
}

/// Get the values for the name
/// Returns a map of values.
///
/// * `name` - a name. e.g. `hello.icp`
#[query(name = "get_record_value")]
#[candid_method(query, rename = "get_record_value")]
fn get_record_value(name: String) -> GetRecordValueResponse {
    let service = ResolverService::new();
    let result = service.get_record_value(name.as_str());
    GetRecordValueResponse::new(result)
}

#[derive(CandidType)]
pub enum GetRecordValueResponse {
    Ok(HashMap<String, String>),
    Err(ErrorInfo),
}

impl GetRecordValueResponse {
    pub fn new(result: ICNSResult<HashMap<String, String>>) -> Self {
        match result {
            Ok(values) => GetRecordValueResponse::Ok(values),
            Err(err) => GetRecordValueResponse::Err(err.into()),
        }
    }
}

candid::export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
#[candid_method(query, rename = "__get_candid_interface_tmp_hack")]
fn __export_did_tmp_() -> String {
    __export_service()
}

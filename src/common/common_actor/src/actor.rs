use crate::state::{State, STATE};
use crate::stats_service::{Stats, StatsService};
use candid::candid_method;
use common::constants::is_dev_env;
use common::dto::{
    from_state_export_data, to_state_export_data, GetStatsResponse, LoadStateRequest,
    StateExportResponse,
};
use common::errors::{BooleanActorResponse, ErrorInfo, NamingError, ServiceResult};
use common::named_principals::PRINCIPAL_NAME_STATE_EXPORTER;
use common::permissions::{must_be_named_principal, must_be_system_owner};
use common::state::StableState;
use ic_cdk::api;
use ic_cdk_macros::*;
use log::{debug, error, info};
use std::collections::HashMap;

#[query(name = "get_stats")]
#[candid_method(query, rename = "get_stats")]
pub fn get_stats() -> GetStatsResponse<Stats> {
    let now = api::time();
    let service = StatsService::default();
    let stats = service.get_stats(now);
    GetStatsResponse::new(Ok(stats))
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

#[update(name = "load_state")]
#[candid_method(update, rename = "load_state")]
pub async fn load_state(request: LoadStateRequest) -> BooleanActorResponse {
    if !is_dev_env() {
        return BooleanActorResponse::new(Err(NamingError::Unknown));
    }
    debug!("load_state: {}", request);
    let caller = &api::caller();
    if must_be_system_owner(caller).is_err() {
        error!("load_state: caller is not system owner");
        return BooleanActorResponse::new(Err(NamingError::PermissionDenied));
    }
    STATE.with(|s| {
        let bytes = from_state_export_data(request);
        let result = State::decode(bytes);
        if result.is_err() {
            error!("Failed to decode state: {:?}", result.err());
            return BooleanActorResponse::Err(ErrorInfo::from(NamingError::Unknown));
        }
        let new_state = result.unwrap();
        s.replace(new_state);
        info!("load_state: success");
        return BooleanActorResponse::Ok(true);
    })
}

#[query(name = "get_wasm_info")]
#[candid_method(query)]
fn get_wasm_info() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    map.insert("VERGEN_BUILD_TIMESTAMP", env!("VERGEN_BUILD_TIMESTAMP"));
    map.insert("VERGEN_BUILD_SEMVER", env!("VERGEN_BUILD_SEMVER"));
    map.insert("VERGEN_GIT_BRANCH", env!("VERGEN_GIT_BRANCH"));
    map.insert(
        "VERGEN_GIT_COMMIT_TIMESTAMP",
        env!("VERGEN_GIT_COMMIT_TIMESTAMP"),
    );
    map.insert("VERGEN_GIT_SEMVER", env!("VERGEN_GIT_SEMVER"));
    map.insert("VERGEN_GIT_SHA", env!("VERGEN_GIT_SHA"));
    map
}

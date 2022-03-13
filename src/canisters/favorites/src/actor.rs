use candid::candid_method;
use candid::CandidType;
use ic_cdk::api;
use ic_cdk_macros::*;
use log::{debug, error, info};

use common::dto::{
    from_state_export_data, to_state_export_data, LoadStateRequest, StateExportResponse,
};
use common::errors::{BooleanActorResponse, ErrorInfo, ICNSError, ICNSResult};
use common::named_principals::PRINCIPAL_NAME_STATE_EXPORTER;
use common::permissions::{must_be_named_principal, must_be_system_owner};
use common::state::StableState;

use crate::service::{ManagerService, Stats};
use crate::state::{State, STATE};

#[query(name = "get_stats")]
#[candid_method(query, rename = "get_stats")]
pub fn get_stats() -> GetStatsResponse {
    let service = ManagerService::new();
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

#[query(name = "get_favorites")]
#[candid_method(query, rename = "get_favorites")]
fn get_favorites() -> GetFavoritesResponse {
    let user = api::caller();
    let service = ManagerService::new();
    let result = service.get_favorites(&user);
    GetFavoritesResponse::new(result)
}

#[derive(CandidType)]
pub enum GetFavoritesResponse {
    Ok(Vec<String>),
    Err(ErrorInfo),
}

impl GetFavoritesResponse {
    pub fn new(result: ICNSResult<Vec<String>>) -> GetFavoritesResponse {
        match result {
            Ok(data) => GetFavoritesResponse::Ok(data),
            Err(err) => GetFavoritesResponse::Err(err.into()),
        }
    }
}

#[update(name = "add_favorite")]
#[candid_method(update, rename = "add_favorite")]
async fn add_favorite(new_item: String) -> BooleanActorResponse {
    let user = api::caller();
    let now = api::time();
    let service = ManagerService::new();
    let result = service.add_favorite(now, &user, &new_item);
    BooleanActorResponse::new(result)
}

#[update(name = "remove_favorite")]
#[candid_method(update, rename = "remove_favorite")]
async fn remove_favorite(item: String) -> BooleanActorResponse {
    let user = api::caller();
    let now = api::time();
    let service = ManagerService::new();
    let result = service.remove_favorite(now, &user, &item);
    BooleanActorResponse::new(result)
}

candid::export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
#[candid_method(query, rename = "__get_candid_interface_tmp_hack")]
fn __export_did_tmp_() -> String {
    __export_service()
}

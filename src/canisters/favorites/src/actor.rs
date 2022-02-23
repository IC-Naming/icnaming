use candid::candid_method;
use candid::CandidType;
use common::dto::{to_state_export_data, StateExportResponse};
use ic_cdk::api;
use ic_cdk_macros::*;

use common::errors::{BooleanActorResponse, ErrorInfo, ICNSActorResult, ICNSResult};
use common::permissions::must_be_system_owner;
use common::state::StableState;

use crate::service::{ManagerService, Stats};
use crate::state::STATE;

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
    let permission_result = must_be_system_owner(caller);
    if permission_result.is_err() {
        return StateExportResponse::new(Err(permission_result.err().unwrap()));
    }

    let source_data = STATE.with(|state| to_state_export_data(state.encode()));
    StateExportResponse::new(Ok(source_data))
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

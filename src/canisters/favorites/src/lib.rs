mod http;
mod service;
mod state;
mod user_favorite_store;

#[path = "../../../common/common_actor/src/actor.rs"]
mod shared_actor;
mod stats_service;

use crate::state::InitArgs;
use common::dto::*;
use common::http::*;
use stats_service::*;
use std::collections::HashMap;

use candid::candid_method;
use candid::CandidType;

use ic_cdk::api;
use ic_cdk_macros::*;

use common::errors::{BooleanActorResponse, ErrorInfo, ServiceResult};

use crate::service::ManagerService;

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
    pub fn new(result: ServiceResult<Vec<String>>) -> GetFavoritesResponse {
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

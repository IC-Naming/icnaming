use candid::candid_method;
use ic_cdk_macros::*;
use ic_kit::ic;

use common::errors::{to_actor_result, ICNSActorResult};

use crate::service::ManagerService;

#[query(name = "get_favorites")]
#[candid_method(query, rename = "get_favorites")]
fn get_favorites() -> ICNSActorResult<Vec<String>> {
    let user = ic::caller();
    let service = ManagerService::new();
    let result = service.get_favorites(&user);
    to_actor_result(result)
}

#[update(name = "add_favorite")]
#[candid_method(update, rename = "add_favorite")]
async fn add_favorite(new_item: String) -> ICNSActorResult<bool> {
    let user = ic::caller();
    let service = ManagerService::new();
    let result = service.add_favorite(&user, &new_item);
    to_actor_result(result)
}

#[update(name = "remove_favorite")]
#[candid_method(update, rename = "remove_favorite")]
async fn remove_favorite(item: String) -> ICNSActorResult<bool> {
    let user = ic::caller();
    let service = ManagerService::new();
    let result = service.remove_favorite(&user, &item);
    to_actor_result(result)
}

candid::export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
#[candid_method(query, rename = "__get_candid_interface_tmp_hack")]
fn __export_did_tmp_() -> String {
    __export_service()
}

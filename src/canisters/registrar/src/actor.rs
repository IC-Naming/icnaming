use candid::{candid_method, Principal};
use ic_cdk::api;
use ic_cdk_macros::*;
use log::debug;

use common::constants::CANISTER_NAME_ENS_ACTIVITY_CLIENT;
use common::dto::{GetPageInput, GetPageOutput};
use common::errors::{to_actor_result, ICNSActorResult, ICNSError};
use common::ic_api::ic_caller;
use common::state::is_principal;

use crate::models::*;
use crate::service::RegistrarService;

#[query(name = "available")]
#[candid_method(query, rename = "available")]
fn available(name: String) -> ICNSActorResult<bool> {
    let service = RegistrarService::new();
    let result = service.available(&name);
    to_actor_result(result)
}

#[query(name = "get_name_expires")]
#[candid_method(query, rename = "get_name_expires")]
fn get_name_expires(name: String) -> ICNSActorResult<u64> {
    let service = RegistrarService::new();
    let result = service.get_name_expires(&name);
    to_actor_result(result)
}

#[update(name = "register_for")]
#[candid_method(update, rename = "register_for")]
async fn register_for(name: String, owner: Principal, years: u64) -> ICNSActorResult<bool> {
    let caller = &ic_caller();
    debug!("register_for: caller: {}", caller);
    if !is_principal(CANISTER_NAME_ENS_ACTIVITY_CLIENT, caller) {
        return to_actor_result(Err(ICNSError::PermissionDenied));
    };

    let mut service = RegistrarService::new();
    let result = service
        .register(
            &name,
            &owner,
            years,
            api::time() / 1_000_000,
            caller,
            QuotaType::LenGte(4),
        )
        .await;
    to_actor_result(result)
}

#[update(name = "register_with_quota")]
#[candid_method(update, rename = "register_with_quota")]
async fn register_with_quota(name: String, quota_type: QuotaType) -> ICNSActorResult<bool> {
    let caller = &ic_caller();
    debug!("register_with_quota: caller: {}", caller);

    let mut service = RegistrarService::new();
    let years = 1;
    let result = service
        .register(
            &name,
            &caller,
            years,
            api::time() / 1_000_000,
            &caller,
            quota_type,
        )
        .await;
    to_actor_result(result)
}

#[query(name = "get_names")]
#[candid_method(query, rename = "get_names")]
fn get_names(
    owner: Principal,
    input: GetPageInput,
) -> ICNSActorResult<GetPageOutput<RegistrationDto>> {
    let service = RegistrarService::new();
    let result = service.get_names(&owner, &input);
    to_actor_result(result)
}

#[query(name = "get_owner")]
#[candid_method(query, rename = "get_owner")]
fn get_owner(name: String) -> ICNSActorResult<Principal> {
    let service = RegistrarService::new();
    let result = service.get_owner(&name);
    to_actor_result(result)
}

#[query(name = "get_details")]
#[candid_method(query, rename = "get_details")]
fn get_details(name: String) -> ICNSActorResult<RegistrationDetails> {
    let service = RegistrarService::new();
    let result = service.get_details(&name);
    to_actor_result(result)
}

#[update(name = "add_quota")]
#[candid_method(update, rename = "add_quota")]
fn add_quota(quota_owner: Principal, quota_type: QuotaType, diff: u32) -> ICNSActorResult<bool> {
    let caller = &ic_caller();
    debug!("add_quota: caller: {}", caller);

    let mut service = RegistrarService::new();
    let result = service.add_quota(caller, quota_owner, quota_type, diff);
    to_actor_result(result)
}

#[update(name = "sub_quota")]
#[candid_method(update, rename = "sub_quota")]
fn sub_quota(quota_owner: Principal, quota_type: QuotaType, diff: u32) -> ICNSActorResult<bool> {
    let caller = &ic_caller();
    debug!("sub_quota: caller: {}", caller);

    let mut service = RegistrarService::new();
    let result = service.sub_quota(caller, quota_owner, quota_type, diff);
    to_actor_result(result)
}

#[query(name = "get_quota")]
#[candid_method(query, rename = "get_quota")]
fn get_quota(quota_owner: Principal, quota_type: QuotaType) -> ICNSActorResult<u32> {
    let caller = &ic_caller();
    debug!("sub_quota: caller: {}", caller);

    let service = RegistrarService::new();
    let result = service.get_quota(caller, quota_owner, quota_type);
    to_actor_result(result)
}

candid::export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
#[candid_method(query, rename = "__get_candid_interface_tmp_hack")]
fn __export_did_tmp_() -> String {
    __export_service()
}

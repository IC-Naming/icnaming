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

/// Check if name is available.
/// Returns true if name is available.
///
/// * `name` - name to check, e.g. "hello.icp"
#[query(name = "available")]
#[candid_method(query, rename = "available")]
pub fn available(name: String) -> ICNSActorResult<bool> {
    let service = RegistrarService::new();
    let result = service.available(&name);
    to_actor_result(result)
}

/// Get expiration date for a name.
/// Returns expiration date.
///
/// * `name` - name to get, e.g. "hello.icp"
#[query(name = "get_name_expires")]
#[candid_method(query, rename = "get_name_expires")]
pub fn get_name_expires(name: String) -> ICNSActorResult<u64> {
    let service = RegistrarService::new();
    let result = service.get_name_expires(&name);
    to_actor_result(result)
}

/// Register a name for a owner. This is private method for activity client.
/// Returns true if name is registered successfully.
///
/// * `name` - name to register, e.g. "hello.icp"
/// * `owner` - owner to register the name for
/// * `years` - number of years to register the name for
#[update(name = "register_for")]
#[candid_method(update, rename = "register_for")]
pub async fn register_for(name: String, owner: Principal, years: u64) -> ICNSActorResult<bool> {
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

/// Register a name for a caller with a quota.
/// Returns true if name is registered successfully.
///
/// * `name` - name to register, e.g. "hello.icp"
/// * `quota_type` - quota type to use
#[update(name = "register_with_quota")]
#[candid_method(update, rename = "register_with_quota")]
pub async fn register_with_quota(name: String, quota_type: QuotaType) -> ICNSActorResult<bool> {
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

/// Get names for a owner.
/// Returns names for a owner.
///
/// * `owner` - owner to get names for
/// * `page` - page offset and limit
#[query(name = "get_names")]
#[candid_method(query, rename = "get_names")]
pub fn get_names(
    owner: Principal,
    input: GetPageInput,
) -> ICNSActorResult<GetPageOutput<RegistrationDto>> {
    let service = RegistrarService::new();
    let result = service.get_names(&owner, &input);
    to_actor_result(result)
}

/// get owner for a name.
/// Returns owner for a name.
///
/// * `name` - name to get owner for
#[query(name = "get_owner")]
#[candid_method(query, rename = "get_owner")]
pub fn get_owner(name: String) -> ICNSActorResult<Principal> {
    let service = RegistrarService::new();
    let result = service.get_owner(&name);
    to_actor_result(result)
}

/// Get details for a name.
/// Returns details for a name.
///
/// * `name` - name to get details for
#[query(name = "get_details")]
#[candid_method(query, rename = "get_details")]
pub fn get_details(name: String) -> ICNSActorResult<RegistrationDetails> {
    let service = RegistrarService::new();
    let result = service.get_details(&name);
    to_actor_result(result)
}

/// Get all details
/// Returns all name details.
///
/// * `name` - name to get details for
#[query(name = "get_all_details")]
#[candid_method(query, rename = "get_all_details")]
pub fn get_all_details(input: GetPageInput) -> ICNSActorResult<Vec<RegistrationDetails>> {
    let caller = api::caller();
    let service = RegistrarService::new();
    let result = service.get_all_details(&caller, &input);
    to_actor_result(result)
}

/// Add quotas to a quota owner.
/// Returns true if quotas are added successfully.
///
/// * `quota_owner` - owner to add quotas to
/// * `quota_type` - quota type to add
/// * `diff` - number of quotas to add
#[update(name = "add_quota")]
#[candid_method(update, rename = "add_quota")]
pub fn add_quota(
    quota_owner: Principal,
    quota_type: QuotaType,
    diff: u32,
) -> ICNSActorResult<bool> {
    let caller = &ic_caller();
    debug!("add_quota: caller: {}", caller);

    let mut service = RegistrarService::new();
    let result = service.add_quota(caller, quota_owner, quota_type, diff);
    to_actor_result(result)
}

/// Remove quotas from a quota owner.
/// Returns true if quotas are removed successfully.
///
/// * `quota_owner` - owner to remove quotas from
/// * `quota_type` - quota type to remove
/// * `diff` - number of quotas to remove
#[update(name = "sub_quota")]
#[candid_method(update, rename = "sub_quota")]
pub fn sub_quota(
    quota_owner: Principal,
    quota_type: QuotaType,
    diff: u32,
) -> ICNSActorResult<bool> {
    let caller = &ic_caller();
    debug!("sub_quota: caller: {}", caller);

    let mut service = RegistrarService::new();
    let result = service.sub_quota(caller, quota_owner, quota_type, diff);
    to_actor_result(result)
}

/// Get quotas for a quota owner.
/// Returns quotas for a quota owner.
///
/// * `quota_owner` - owner to get quotas for
/// * `quota_type` - quota type to get
#[query(name = "get_quota")]
#[candid_method(query, rename = "get_quota")]
pub fn get_quota(quota_owner: Principal, quota_type: QuotaType) -> ICNSActorResult<u32> {
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

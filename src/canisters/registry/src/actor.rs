use candid::{candid_method, Principal};
use ic_cdk_macros::*;

use common::constants::CANISTER_NAME_REGISTRAR;
use common::dto::*;
use common::errors::{to_actor_result, ICNSActorResult, ICNSError};
use common::ic_api::ic_caller;
use common::state::{get_principal, is_owner};

use crate::service::RegistriesService;

/// TODO remove and move it to init
#[update(name = "set_top_name")]
#[candid_method(update, rename = "set_top_name")]
fn set_top_name() -> ICNSActorResult<bool> {
    if !is_owner(&ic_caller()) {
        return Err(ICNSError::OwnerOnly.into());
    }
    let registrar = get_principal(&CANISTER_NAME_REGISTRAR).unwrap();
    let mut service = RegistriesService::new();
    let result = service.set_top_icp_name(registrar);
    to_actor_result(result)
}

/// Set owner of subdomain
/// Returns details of the new registry
///
/// * `label` - label of subdomain. e.g. `www`
/// * `parent_name` - parent name of subdomain. e.g. `hello.icp`
/// * `sub_owner` - owner of subdomain
/// * `ttl` - ttl of subdomain
/// * `resolver` - resolver of subdomain
#[update(name = "set_subdomain_owner")]
#[candid_method(update, rename = "set_subdomain_owner")]
async fn set_subdomain_owner(
    label: String,
    parent_name: String,
    sub_owner: Principal,
    ttl: u64,
    resolver: Principal,
) -> ICNSActorResult<RegistryDto> {
    let owner = ic_caller();
    let mut service = RegistriesService::new();
    let result = service
        .set_subdomain_owner(label, parent_name, owner, sub_owner, ttl, resolver)
        .await;
    to_actor_result(result)
}

/// Set full info of subdomain
/// Returns true if success
///
/// * `name` - a name. e.g. `hello.icp`
/// * `ttl` - ttl of name
/// * `resolver` - resolver of name
#[update(name = "set_record")]
#[candid_method(update, rename = "set_record")]
fn set_record(name: String, ttl: u64, resolver: Principal) -> ICNSActorResult<bool> {
    let caller = ic_caller();
    let mut service = RegistriesService::new();
    let result = service.set_record(&caller, name.as_str(), ttl, &resolver);
    to_actor_result(result)
}

/// Get resolver
///
/// * `name` - a name. e.g. `hello.icp`
#[query(name = "get_resolver")]
#[candid_method(query, rename = "get_resolver")]
fn get_resolver(name: String) -> ICNSActorResult<Principal> {
    let service = RegistriesService::new();
    let result = service.get_resolver(&name);
    to_actor_result(result)
}

/// Set approval status of operator. Operator can be update info of subdomain.
/// Returns true if success.
///
/// * `name` - a name. e.g. `hello.icp`
/// * `operator` - operator to be set.
/// * `approved` - approval status of operator
#[update(name = "set_approval")]
#[candid_method(update, rename = "set_approval")]
fn set_approval(name: String, operator: Principal, approved: bool) -> ICNSActorResult<bool> {
    let mut service = RegistriesService::new();
    let caller = &ic_caller();
    let result = if approved {
        service.set_approval(name.as_str(), caller, &operator)
    } else {
        service.remove_approval(name.as_str(), caller, &operator)
    };
    to_actor_result(result)
}

/// Get name that owned by owner
/// Returns list of names owned by owner
///
/// * `owner` - owner of names
/// * `page` - page offset and limit
#[query(name = "get_controlled_names")]
#[candid_method(query, rename = "get_controlled_names")]
fn get_controlled_names(
    owner: Principal,
    page: GetPageInput,
) -> ICNSActorResult<GetPageOutput<String>> {
    let service = RegistriesService::new();
    let result = service.get_controlled_names(owner, page);
    to_actor_result(result)
}

/// Get owner and operators of name
///
/// * `name` - a name. e.g. `hello.icp`
#[query(name = "get_users")]
#[candid_method(query, rename = "get_users")]
fn get_users(name: String) -> ICNSActorResult<RegistryUsers> {
    let service = RegistriesService::new();
    let result = service.get_users(&name);
    to_actor_result(result)
}

/// Get owner of name
///
/// * `name` - a name. e.g. `hello.icp`
#[query(name = "get_owner")]
#[candid_method(query, rename = "get_owner")]
fn get_owner(name: String) -> ICNSActorResult<Principal> {
    let service = RegistriesService::new();
    let result = service.get_owner(&name);
    to_actor_result(result)
}

/// Get ttl of name
///
/// * `name` - a name. e.g. `hello.icp`
#[query(name = "get_ttl")]
#[candid_method(query, rename = "get_ttl")]
fn get_ttl(name: String) -> ICNSActorResult<u64> {
    let service = RegistriesService::new();
    let result = service.get_ttl(&name);
    to_actor_result(result)
}

/// Get details of name
///
/// * `name` - a name. e.g. `hello.icp`
#[query(name = "get_details")]
#[candid_method(query, rename = "get_details")]
fn get_details(name: String) -> ICNSActorResult<RegistryDto> {
    let service = RegistriesService::new();
    let result = service.get_details(&name);
    to_actor_result(result)
}

candid::export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
#[candid_method(query, rename = "__get_candid_interface_tmp_hack")]
fn __export_did_tmp_() -> String {
    __export_service()
}

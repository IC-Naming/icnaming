mod http;
mod registry_store;
mod service;
mod state;

#[path = "../../../common/common_actor/src/actor.rs"]
mod shared_actor;
mod stats_service;

use crate::state::InitArgs;
use candid::{candid_method, CandidType, Principal};
use common::dto::*;
use common::errors::{BooleanActorResponse, ErrorInfo, NamingError, ServiceResult};
use common::http::*;
use common::named_canister_ids::{get_named_get_canister_id, CanisterNames};
use ic_cdk_macros::*;
use stats_service::*;
use std::collections::HashMap;

use crate::service::RegistriesService;

/// Set owner of subdomain
/// Returns details of the new registry
///
/// * `label` - label of subdomain. e.g. `www`
/// * `parent_name` - parent name of subdomain. e.g. `hello.ic`
/// * `sub_owner` - owner of subdomain
/// * `ttl` - ttl of subdomain
/// * `resolver` - resolver of subdomain
#[update(name = "set_subdomain_owner")]
#[candid_method(update)]
async fn set_subdomain_owner(
    label: String,
    parent_name: String,
    sub_owner: Principal,
    ttl: u64,
    resolver: Principal,
) -> SetSubdomainOwnerResponse {
    let owner = ic_cdk::api::caller();
    // TODO: to enable add subdomain to normal user but need to limit count of subdomains
    let result = if owner != get_named_get_canister_id(CanisterNames::Registrar) {
        Err(NamingError::PermissionDenied)
    } else {
        let mut service = RegistriesService::new();
        service
            .set_subdomain_owner(label, parent_name, owner, sub_owner, ttl, resolver)
            .await
    };
    SetSubdomainOwnerResponse::new(result)
}

#[derive(CandidType)]
pub enum SetSubdomainOwnerResponse {
    Ok(RegistryDto),
    Err(ErrorInfo),
}

impl SetSubdomainOwnerResponse {
    pub fn new(result: ServiceResult<RegistryDto>) -> SetSubdomainOwnerResponse {
        match result {
            Ok(data) => SetSubdomainOwnerResponse::Ok(data),
            Err(err) => SetSubdomainOwnerResponse::Err(err.into()),
        }
    }
}

/// Set full info of subdomain
/// Returns true if success
///
/// * `name` - a name. e.g. `hello.ic`
/// * `ttl` - ttl of name
/// * `resolver` - resolver of name
#[update(name = "set_record")]
#[candid_method(update)]
fn set_record(name: String, ttl: u64, resolver: Principal) -> BooleanActorResponse {
    let caller = ic_cdk::api::caller();
    let mut service = RegistriesService::new();
    let result = service.set_record(&caller, name.as_str(), ttl, &resolver);
    BooleanActorResponse::new(result)
}

/// Get resolver
///
/// * `name` - a name. e.g. `hello.ic`
#[query(name = "get_resolver")]
#[candid_method(query)]
fn get_resolver(name: String) -> GetResolverResponse {
    let service = RegistriesService::new();
    let result = service.get_resolver(&name);
    GetResolverResponse::new(result)
}

#[derive(CandidType)]
pub enum GetResolverResponse {
    Ok(Principal),
    Err(ErrorInfo),
}

impl GetResolverResponse {
    pub fn new(result: ServiceResult<Principal>) -> GetResolverResponse {
        match result {
            Ok(data) => GetResolverResponse::Ok(data),
            Err(err) => GetResolverResponse::Err(err.into()),
        }
    }
}

#[update(name = "set_resolver")]
#[candid_method(update)]
fn set_resolver(name: String, resolver: Principal) -> BooleanActorResponse {
    let service = RegistriesService::new();
    let caller = ic_cdk::api::caller();

    let result = service.set_resolver(caller, &name, resolver);
    BooleanActorResponse::new(result)
}

/// Set approval status of operator. Operator can be update info of subdomain.
/// Returns true if success.
///
/// * `name` - a name. e.g. `hello.ic`
/// * `operator` - operator to be set.
/// * `approved` - approval status of operator
#[update(name = "set_approval")]
#[candid_method(update)]
fn set_approval(name: String, operator: Principal, approved: bool) -> BooleanActorResponse {
    let mut service = RegistriesService::new();
    let caller = &ic_cdk::api::caller();
    let result = if approved {
        service.set_approval(name.as_str(), caller, &operator)
    } else {
        service.remove_approval(name.as_str(), caller, &operator)
    };
    BooleanActorResponse::new(result)
}

/// Get name that owned by owner
/// Returns list of names owned by owner
///
/// * `owner` - owner of names
/// * `page` - page offset and limit
#[query(name = "get_controlled_names")]
#[candid_method(query)]
fn get_controlled_names(owner: Principal, page: GetPageInput) -> GetControlledNamesResponse {
    let service = RegistriesService::new();
    let result = service.get_controlled_names(owner, page);
    GetControlledNamesResponse::new(result)
}

#[derive(CandidType)]
pub enum GetControlledNamesResponse {
    Ok(GetPageOutput<String>),
    Err(ErrorInfo),
}

impl GetControlledNamesResponse {
    pub fn new(result: ServiceResult<GetPageOutput<String>>) -> GetControlledNamesResponse {
        match result {
            Ok(data) => GetControlledNamesResponse::Ok(data),
            Err(err) => GetControlledNamesResponse::Err(err.into()),
        }
    }
}

/// Get owner and operators of name
///
/// * `name` - a name. e.g. `hello.ic`
#[query(name = "get_users")]
#[candid_method(query)]
fn get_users(name: String) -> GetUsersResponse {
    let service = RegistriesService::new();
    let result = service.get_users(&name);
    GetUsersResponse::new(result)
}

#[derive(CandidType)]
pub enum GetUsersResponse {
    Ok(RegistryUsers),
    Err(ErrorInfo),
}

impl GetUsersResponse {
    pub fn new(result: ServiceResult<RegistryUsers>) -> GetUsersResponse {
        match result {
            Ok(data) => GetUsersResponse::Ok(data),
            Err(err) => GetUsersResponse::Err(err.into()),
        }
    }
}

/// Get owner of name
///
/// * `name` - a name. e.g. `hello.ic`
#[query(name = "get_owner")]
#[candid_method(query)]
fn get_owner(name: String) -> GetOwnerResponse {
    let service = RegistriesService::new();
    let result = service.get_owner(&name);
    GetOwnerResponse::new(result)
}

#[derive(CandidType)]
pub enum GetOwnerResponse {
    Ok(Principal),
    Err(ErrorInfo),
}

impl GetOwnerResponse {
    pub fn new(result: ServiceResult<Principal>) -> GetOwnerResponse {
        match result {
            Ok(data) => GetOwnerResponse::Ok(data),
            Err(err) => GetOwnerResponse::Err(err.into()),
        }
    }
}

#[update(name = "set_owner")]
#[candid_method(update)]
fn set_owner(name: String, new_owner: Principal) -> BooleanActorResponse {
    let caller = ic_cdk::api::caller();
    let service = RegistriesService::new();
    let result = service.set_owner(caller, name.as_str(), new_owner);
    BooleanActorResponse::new(result)
}

/// Get ttl of name
///
/// * `name` - a name. e.g. `hello.ic`
#[query(name = "get_ttl")]
#[candid_method(query)]
fn get_ttl(name: String) -> GetTtlResponse {
    let service = RegistriesService::new();
    let result = service.get_ttl(&name);
    GetTtlResponse::new(result)
}

#[derive(CandidType)]
pub enum GetTtlResponse {
    Ok(u64),
    Err(ErrorInfo),
}

impl GetTtlResponse {
    pub fn new(result: ServiceResult<u64>) -> GetTtlResponse {
        match result {
            Ok(data) => GetTtlResponse::Ok(data),
            Err(err) => GetTtlResponse::Err(err.into()),
        }
    }
}

/// Get details of name
///
/// * `name` - a name. e.g. `hello.ic`
#[query(name = "get_details")]
#[candid_method(query)]
fn get_details(name: String) -> GetDetailsResponse {
    let service = RegistriesService::new();
    let result = service.get_details(&name);
    GetDetailsResponse::new(result)
}

#[derive(CandidType)]
pub enum GetDetailsResponse {
    Ok(RegistryDto),
    Err(ErrorInfo),
}

impl GetDetailsResponse {
    pub fn new(result: ServiceResult<RegistryDto>) -> GetDetailsResponse {
        match result {
            Ok(data) => GetDetailsResponse::Ok(data),
            Err(err) => GetDetailsResponse::Err(err.into()),
        }
    }
}

#[update(name = "reclaim_name")]
#[candid_method(update)]
fn reclaim_name(name: String, owner: Principal, resolver: Principal) -> BooleanActorResponse {
    let caller = &ic_cdk::api::caller();

    let mut service = RegistriesService::new();
    let result = service.reclaim_name(name.as_str(), caller, &owner, &resolver);
    BooleanActorResponse::new(result)
}

#[update(name = "transfer")]
#[candid_method(update)]
async fn transfer(name: String, new_owner: Principal, resolver: Principal) -> BooleanActorResponse {
    let caller = &ic_cdk::api::caller();

    let mut service = RegistriesService::new();
    let result = service
        .transfer(name.as_str(), caller, &new_owner, resolver)
        .await;
    BooleanActorResponse::new(result)
}

candid::export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
#[candid_method(query)]
fn __export_did_tmp_() -> String {
    __export_service()
}

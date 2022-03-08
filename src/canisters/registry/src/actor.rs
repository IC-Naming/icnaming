use candid::{candid_method, CandidType, Principal};
use ic_cdk::api;
use ic_cdk_macros::*;

use common::dto::*;
use common::errors::{BooleanActorResponse, ErrorInfo, ICNSError, ICNSResult};
use common::ic_api::ic_caller;
use common::named_canister_ids::get_named_get_canister_id;
use common::named_canister_ids::CANISTER_NAME_REGISTRAR;
use common::named_principals::PRINCIPAL_NAME_STATE_EXPORTER;
use common::permissions::{must_be_named_principal, must_be_system_owner};
use common::state::StableState;

use crate::service::{RegistriesService, Stats};
use crate::state::STATE;

#[query(name = "get_stats")]
#[candid_method(query, rename = "get_stats")]
pub fn get_stats() -> GetStatsResponse {
    let service = RegistriesService::new();
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
) -> SetSubdomainOwnerResponse {
    let owner = ic_caller();
    // TODO: to enable add subdomain to normal user but need to limit count of subdomains
    let result = if owner != get_named_get_canister_id(CANISTER_NAME_REGISTRAR) {
        Err(ICNSError::PermissionDenied)
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
    pub fn new(result: ICNSResult<RegistryDto>) -> SetSubdomainOwnerResponse {
        match result {
            Ok(data) => SetSubdomainOwnerResponse::Ok(data),
            Err(err) => SetSubdomainOwnerResponse::Err(err.into()),
        }
    }
}

/// Set full info of subdomain
/// Returns true if success
///
/// * `name` - a name. e.g. `hello.icp`
/// * `ttl` - ttl of name
/// * `resolver` - resolver of name
#[update(name = "set_record")]
#[candid_method(update, rename = "set_record")]
fn set_record(name: String, ttl: u64, resolver: Principal) -> BooleanActorResponse {
    let caller = ic_caller();
    let mut service = RegistriesService::new();
    let result = service.set_record(&caller, name.as_str(), ttl, &resolver);
    BooleanActorResponse::new(result)
}

/// Get resolver
///
/// * `name` - a name. e.g. `hello.icp`
#[query(name = "get_resolver")]
#[candid_method(query, rename = "get_resolver")]
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
    pub fn new(result: ICNSResult<Principal>) -> GetResolverResponse {
        match result {
            Ok(data) => GetResolverResponse::Ok(data),
            Err(err) => GetResolverResponse::Err(err.into()),
        }
    }
}

/// Set approval status of operator. Operator can be update info of subdomain.
/// Returns true if success.
///
/// * `name` - a name. e.g. `hello.icp`
/// * `operator` - operator to be set.
/// * `approved` - approval status of operator
#[update(name = "set_approval")]
#[candid_method(update, rename = "set_approval")]
fn set_approval(name: String, operator: Principal, approved: bool) -> BooleanActorResponse {
    let mut service = RegistriesService::new();
    let caller = &ic_caller();
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
#[candid_method(query, rename = "get_controlled_names")]
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
    pub fn new(result: ICNSResult<GetPageOutput<String>>) -> GetControlledNamesResponse {
        match result {
            Ok(data) => GetControlledNamesResponse::Ok(data),
            Err(err) => GetControlledNamesResponse::Err(err.into()),
        }
    }
}

/// Get owner and operators of name
///
/// * `name` - a name. e.g. `hello.icp`
#[query(name = "get_users")]
#[candid_method(query, rename = "get_users")]
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
    pub fn new(result: ICNSResult<RegistryUsers>) -> GetUsersResponse {
        match result {
            Ok(data) => GetUsersResponse::Ok(data),
            Err(err) => GetUsersResponse::Err(err.into()),
        }
    }
}

/// Get owner of name
///
/// * `name` - a name. e.g. `hello.icp`
#[query(name = "get_owner")]
#[candid_method(query, rename = "get_owner")]
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
    pub fn new(result: ICNSResult<Principal>) -> GetOwnerResponse {
        match result {
            Ok(data) => GetOwnerResponse::Ok(data),
            Err(err) => GetOwnerResponse::Err(err.into()),
        }
    }
}

/// Get ttl of name
///
/// * `name` - a name. e.g. `hello.icp`
#[query(name = "get_ttl")]
#[candid_method(query, rename = "get_ttl")]
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
    pub fn new(result: ICNSResult<u64>) -> GetTtlResponse {
        match result {
            Ok(data) => GetTtlResponse::Ok(data),
            Err(err) => GetTtlResponse::Err(err.into()),
        }
    }
}

/// Get details of name
///
/// * `name` - a name. e.g. `hello.icp`
#[query(name = "get_details")]
#[candid_method(query, rename = "get_details")]
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
    pub fn new(result: ICNSResult<RegistryDto>) -> GetDetailsResponse {
        match result {
            Ok(data) => GetDetailsResponse::Ok(data),
            Err(err) => GetDetailsResponse::Err(err.into()),
        }
    }
}

#[update(name = "reclaim_name")]
#[candid_method(update, rename = "reclaim_name")]
fn reclaim_name(name: String, owner: Principal, resolver: Principal) -> BooleanActorResponse {
    let caller = &ic_caller();

    let mut service = RegistriesService::new();
    let result = service.reclaim_name(name.as_str(), caller, &owner, &resolver);
    BooleanActorResponse::new(result)
}

candid::export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
#[candid_method(query, rename = "__get_candid_interface_tmp_hack")]
fn __export_did_tmp_() -> String {
    __export_service()
}

mod coinaddress;
mod http;
mod resolver_store;
mod service;
mod set_record_value_input;
mod state;

mod reverse_resolver_store;
#[path = "../../../common/common_actor/src/actor.rs"]
mod shared_actor;
mod stats_service;

use common::dto::*;
use common::http::*;
use stats_service::*;
use std::collections::HashMap;

use candid::{candid_method, CandidType, Principal};
use common::CallContext;

use ic_cdk_macros::*;

use common::errors::{BooleanActorResponse, ErrorInfo, ServiceResult};
use common::named_canister_ids::CanisterNames;

use crate::service::{ImportRecordValueRequest, ResolverService};

use crate::state::InitArgs;

/// Ensure the resolver is created.
/// Returns true if the resolver is created, false otherwise.
///
/// * `name` - a name. e.g. `hello.ic`
#[update(name = "ensure_resolver_created")]
#[candid_method(update)]
fn ensure_resolver_created(name: String) -> BooleanActorResponse {
    let call_context = CallContext::from_ic();
    let result = if let Err(e) = call_context.must_be_named_canister(CanisterNames::Registry) {
        Err(e)
    } else {
        let mut service = ResolverService::default();
        service.ensure_resolver_created(name.as_str())
    };
    BooleanActorResponse::new(result)
}

/// Set the record values for the name
/// Returns true if the record is set, false otherwise.
///
/// * `name` - a name. e.g. `hello.ic`
/// * `values` - a list of values. e.g. `canister.ic`
#[update(name = "set_record_value")]
#[candid_method(update)]
async fn set_record_value(
    name: String,
    patch_values: HashMap<String, String>,
) -> BooleanActorResponse {
    let call_context = CallContext::from_ic();
    let mut service = ResolverService::default();
    let result = service
        .set_record_value(call_context, &name, patch_values)
        .await;
    BooleanActorResponse::new(result)
}

/// Get the values for the name
/// Returns a map of values.
///
/// * `name` - a name. e.g. `hello.ic`
#[query(name = "get_record_value")]
#[candid_method(query)]
fn get_record_value(name: String) -> GetRecordValueResponse {
    let service = ResolverService::default();
    let result = service.get_record_value(name.as_str());
    GetRecordValueResponse::new(result)
}

#[derive(CandidType)]
pub enum GetRecordValueResponse {
    Ok(HashMap<String, String>),
    Err(ErrorInfo),
}

impl GetRecordValueResponse {
    pub fn new(result: ServiceResult<HashMap<String, String>>) -> Self {
        match result {
            Ok(values) => GetRecordValueResponse::Ok(values),
            Err(err) => GetRecordValueResponse::Err(err.into()),
        }
    }
}

#[update(name = "remove_resolvers")]
#[candid_method(update)]
fn remove_resolvers(names: Vec<String>) -> BooleanActorResponse {
    let call_context = CallContext::from_ic();
    let service = ResolverService::default();
    let result = service.remove_resolvers(call_context, names);
    BooleanActorResponse::new(result)
}

#[query(name = "reverse_resolve_principal")]
#[candid_method(query)]
fn reverse_resolve_principal(principal: Principal) -> ReverseResolvePrincipalResponse {
    let service = ResolverService::default();
    let result = service.reverse_resolve_principal(principal);
    ReverseResolvePrincipalResponse::new(result)
}

#[derive(CandidType)]
pub enum ReverseResolvePrincipalResponse {
    Ok(Option<String>),
    Err(ErrorInfo),
}

impl ReverseResolvePrincipalResponse {
    pub fn new(result: ServiceResult<Option<String>>) -> Self {
        match result {
            Ok(name) => ReverseResolvePrincipalResponse::Ok(name),
            Err(err) => ReverseResolvePrincipalResponse::Err(err.into()),
        }
    }
}

#[derive(CandidType)]
pub enum BatchGetReverseResolvePrincipalResponse {
    Ok(HashMap<Principal, Option<String>>),
    Err(ErrorInfo),
}

impl BatchGetReverseResolvePrincipalResponse {
    pub fn new(result: ServiceResult<HashMap<Principal, Option<String>>>) -> Self {
        match result {
            Ok(name) => BatchGetReverseResolvePrincipalResponse::Ok(name),
            Err(err) => BatchGetReverseResolvePrincipalResponse::Err(err.into()),
        }
    }
}

#[query(name = "batch_get_reverse_resolve_principal")]
#[candid_method(query)]
fn batch_get_reverse_resolve_principal(
    principals: Vec<Principal>,
) -> BatchGetReverseResolvePrincipalResponse {
    let service = ResolverService::default();
    let result = service.batch_get_reverse_resolve_principal(principals);
    BatchGetReverseResolvePrincipalResponse::new(result)
}

#[update(name = "import_record_value")]
#[candid_method(update)]
async fn import_record_value(request: ImportRecordValueRequest) -> BooleanActorResponse {
    let call_context = CallContext::from_ic();
    let service = ResolverService::default();
    let result = service.import_record_value(&call_context, &request);
    BooleanActorResponse::new(result)
}

candid::export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
#[candid_method(query, rename = "__get_candid_interface_tmp_hack")]
fn __export_did_tmp_() -> String {
    __export_service()
}

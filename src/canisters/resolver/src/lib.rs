mod coinaddress;
mod http;
mod resolver_store;
mod service;
mod state;

#[path = "../../../common/common_actor/src/actor.rs"]
mod shared_actor;
mod stats_service;

use common::dto::*;
use common::http::*;
use stats_service::*;
use std::collections::HashMap;

use candid::{candid_method, CandidType};
use common::CallContext;
use ic_cdk::{api, caller};
use ic_cdk_macros::*;

use common::errors::{BooleanActorResponse, ErrorInfo, NamingError, ServiceResult};
use common::named_canister_ids::{get_named_get_canister_id, CanisterNames};

use crate::service::ResolverService;
use crate::state::InitArgs;

/// Ensure the resolver is created.
/// Returns true if the resolver is created, false otherwise.
///
/// * `name` - a name. e.g. `hello.ark`
#[update(name = "ensure_resolver_created")]
#[candid_method(update, rename = "ensure_resolver_created")]
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
/// * `name` - a name. e.g. `hello.ark`
/// * `values` - a list of values. e.g. `canister.ark`
#[update(name = "set_record_value")]
#[candid_method(update, rename = "set_record_value")]
async fn set_record_value(
    name: String,
    patch_values: HashMap<String, String>,
) -> BooleanActorResponse {
    let call_context = CallContext::from_ic();
    let mut service = ResolverService::default();
    let result = service
        .set_record_value(call_context, name.as_str(), patch_values)
        .await;
    BooleanActorResponse::new(result)
}

/// Get the values for the name
/// Returns a map of values.
///
/// * `name` - a name. e.g. `hello.ark`
#[query(name = "get_record_value")]
#[candid_method(query, rename = "get_record_value")]
fn get_record_value(name: String) -> GetRecordValueResponse {
    let service = ResolverService::default();
    let result = service.get_record_value(name.as_str());
    GetRecordValueResponse::new(result)
}

#[update(name = "remove_resolvers")]
#[candid_method(update, rename = "remove_resolvers")]
fn remove_resolvers(names: Vec<String>) -> BooleanActorResponse {
    let call_context = CallContext::from_ic();
    let service = ResolverService::default();
    let result = service.remove_resolvers(call_context, names);
    BooleanActorResponse::new(result)
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

candid::export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
#[candid_method(query, rename = "__get_candid_interface_tmp_hack")]
fn __export_did_tmp_() -> String {
    __export_service()
}

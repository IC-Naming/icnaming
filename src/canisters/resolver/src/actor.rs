use std::collections::HashMap;

use candid::candid_method;
use ic_cdk_macros::*;

use common::errors::{to_actor_result, ICNSActorResult};

use crate::service::ResolverService;

#[update(name = "ensure_resolver_created")]
#[candid_method(update, rename = "ensure_resolver_created")]
fn ensure_resolver_created(name: String) -> ICNSActorResult<bool> {
    let mut service = ResolverService::new();
    let result = service.ensure_resolver_created(name.as_str());
    to_actor_result(result)
}

#[update(name = "set_record_value")]
#[candid_method(update, rename = "set_record_value")]
async fn set_record_value(
    name: String,
    patch_values: HashMap<String, String>,
) -> ICNSActorResult<bool> {
    let mut service = ResolverService::new();
    let result = service.set_record_value(name.as_str(), &patch_values).await;
    to_actor_result(result)
}

#[query(name = "get_record_value")]
#[candid_method(query, rename = "get_record_value")]
fn get_record_value(name: String) -> ICNSActorResult<HashMap<String, String>> {
    let service = ResolverService::new();
    let result = service.get_record_value(name.as_str());
    to_actor_result(result)
}

candid::export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
#[candid_method(query, rename = "__get_candid_interface_tmp_hack")]
fn __export_did_tmp_() -> String {
    __export_service()
}

mod http;
mod service;
mod state;

mod stats_service;

use common::dto::*;
use common::http::*;
use stats_service::*;
use std::collections::HashMap;

use candid::{candid_method, Nat};
use ic_cdk::api;

use ic_cdk_macros::*;

use common::errors::BooleanActorResponse;

use crate::service::ICNamingLedgerService;

#[update(name = "withdraw_icp")]
#[candid_method(update)]
async fn withdraw_icp(sub_account: u8, amount: Nat) -> BooleanActorResponse {
    let caller = api::caller();
    let service = ICNamingLedgerService::default();
    let result = service.withdraw_icp(caller, sub_account, amount).await;
    BooleanActorResponse::new(result)
}

candid::export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
#[candid_method(query)]
fn __export_did_tmp_() -> String {
    __export_service()
}

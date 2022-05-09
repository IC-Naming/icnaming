mod http;
mod name_locker;
mod name_order_store;
mod payment_store;
mod periodic_tasks_runner;
mod quota_import_store;
mod registration_approval_store;
mod registration_store;
mod reserved_list;
mod service;
mod settings;
mod state;
mod user_quota_store;

mod balance_store;
#[path = "../../../common/common_actor/src/actor.rs"]
mod shared_actor;
mod stats_service;
mod token_service;

use crate::state::InitArgs;
use candid::{candid_method, CandidType, Deserialize, Principal};
use common::constants::is_env;
use common::constants::NamingEnv::Production;
use common::dto::*;
use common::http::*;
use ic_cdk::api;
use ic_cdk_macros::*;
use log::debug;
use stats_service::*;
use std::collections::HashMap;

use common::dto::{GetPageInput, GetPageOutput, ImportQuotaRequest, ImportQuotaStatus};
use common::errors::{BooleanActorResponse, ErrorInfo, ServiceResult};
use common::named_principals::PRINCIPAL_NAME_TIMER_TRIGGER;
use common::permissions::must_be_named_principal;
use common::{CallContext, TimeInNs};

use crate::name_order_store::GetNameOrderResponse;
use crate::periodic_tasks_runner::run_periodic_tasks;
use crate::registration_store::{RegistrationDetails, RegistrationDto};
use crate::service::{
    BatchTransferRequest, ImportNameRegistrationRequest, PriceTable, RegistrarService,
    RenewNameRequest, SubmitOrderRequest, SubmitOrderResponse,
};

use crate::user_quota_store::{QuotaType, TransferQuotaDetails};

#[update(name = "run_tasks")]
#[candid_method(update)]
pub async fn run_tasks() -> BooleanActorResponse {
    let caller = &api::caller();
    let permission_result = must_be_named_principal(caller, PRINCIPAL_NAME_TIMER_TRIGGER);
    if permission_result.is_err() {
        return BooleanActorResponse::new(Err(permission_result.err().unwrap()));
    }
    run_periodic_tasks().await;
    BooleanActorResponse::new(Ok(true))
}

#[update(name = "get_price_table")]
#[candid_method(update)]
pub async fn get_price_table() -> GetPriceTableResponse {
    let service = RegistrarService::default();
    let price_table = service.get_price_table().await;
    GetPriceTableResponse::new(price_table)
}

#[derive(CandidType)]
pub enum GetPriceTableResponse {
    Ok(PriceTable),
    Err(ErrorInfo),
}

impl GetPriceTableResponse {
    pub fn new(result: ServiceResult<PriceTable>) -> GetPriceTableResponse {
        match result {
            Ok(price_table) => GetPriceTableResponse::Ok(price_table),
            Err(err) => GetPriceTableResponse::Err(err.into()),
        }
    }
}

#[query(name = "available")]
#[candid_method(query)]
pub fn available(name: String) -> BooleanActorResponse {
    let service = RegistrarService::default();
    match service.available(&name) {
        Ok(_) => BooleanActorResponse::new(Ok(true)),
        Err(err) => BooleanActorResponse::new(Err(err.into())),
    }
}

#[query(name = "get_name_expires")]
#[candid_method(query)]
pub fn get_name_expires(name: String) -> GetNameExpiresActorResponse {
    let service = RegistrarService::default();
    let result = service.get_name_expires(&name);
    GetNameExpiresActorResponse::new(result)
}

#[derive(CandidType)]
pub enum GetNameExpiresActorResponse {
    Ok(u64),
    Err(ErrorInfo),
}

impl GetNameExpiresActorResponse {
    pub fn new(result: ServiceResult<u64>) -> GetNameExpiresActorResponse {
        match result {
            Ok(expires) => GetNameExpiresActorResponse::Ok(expires),
            Err(err) => GetNameExpiresActorResponse::Err(err.into()),
        }
    }
}

#[update(name = "register_for")]
#[candid_method(update)]
pub async fn register_for(name: String, owner: Principal, years: u64) -> BooleanActorResponse {
    let caller = &api::caller();
    debug!("register_for: caller: {}", caller);

    let mut service = RegistrarService::default();
    let result = service
        .register_with_quota(
            name,
            owner,
            years as u32,
            TimeInNs(api::time()),
            caller,
            QuotaType::LenGte(4),
            false,
        )
        .await;
    BooleanActorResponse::new(result)
}

#[update(name = "register_with_quota")]
#[candid_method(update)]
pub async fn register_with_quota(name: String, quota_type: QuotaType) -> BooleanActorResponse {
    let caller = api::caller();
    debug!("register_with_quota: caller: {}", caller);

    let mut service = RegistrarService::default();
    let years = 1;
    let result = service
        .register_with_quota(
            name,
            caller,
            years,
            TimeInNs(api::time()),
            &caller,
            quota_type,
            false,
        )
        .await;
    BooleanActorResponse::new(result)
}

#[update(name = "pay_my_order")]
#[candid_method(update)]
pub async fn pay_my_order() -> BooleanActorResponse {
    let caller = api::caller();
    let now = api::time();
    let service = RegistrarService::default();
    let result = service.pay_my_order(caller, TimeInNs(now)).await;
    BooleanActorResponse::new(result)
}

#[update(name = "register_from_gateway")]
#[candid_method(update)]
pub async fn register_from_gateway(name: String, owner: Principal) -> BooleanActorResponse {
    let caller = api::caller();
    debug!("register_from_gateway: caller: {}", caller);

    let mut service = RegistrarService::default();
    let result = service
        .register_from_gateway(&caller, &name, owner, TimeInNs(api::time()))
        .await;
    BooleanActorResponse::new(result)
}

#[query(name = "get_names")]
#[candid_method(query)]
pub fn get_names(owner: Principal, input: GetPageInput) -> GetNamesActorResponse {
    let service = RegistrarService::default();
    let result = service.get_names(&owner, &input);
    GetNamesActorResponse::new(result)
}

#[derive(CandidType)]
pub enum GetNamesActorResponse {
    Ok(GetPageOutput<RegistrationDto>),
    Err(ErrorInfo),
}

impl GetNamesActorResponse {
    pub fn new(result: ServiceResult<GetPageOutput<RegistrationDto>>) -> GetNamesActorResponse {
        match result {
            Ok(output) => GetNamesActorResponse::Ok(output),
            Err(err) => GetNamesActorResponse::Err(err.into()),
        }
    }
}

#[query(name = "get_owner")]
#[candid_method(query)]
pub fn get_owner(name: String) -> GetOwnerActorResponse {
    let service = RegistrarService::default();
    let result = service.get_owner(&name);
    GetOwnerActorResponse::new(result)
}

#[derive(CandidType)]
pub enum GetOwnerActorResponse {
    Ok(Principal),
    Err(ErrorInfo),
}

impl GetOwnerActorResponse {
    pub fn new(result: ServiceResult<Principal>) -> GetOwnerActorResponse {
        match result {
            Ok(owner) => GetOwnerActorResponse::Ok(owner),
            Err(err) => GetOwnerActorResponse::Err(err.into()),
        }
    }
}

#[query(name = "get_details")]
#[candid_method(query)]
pub fn get_details(name: String) -> GetDetailsActorResponse {
    let service = RegistrarService::default();
    let result = service.get_details(&name);
    GetDetailsActorResponse::new(result)
}

#[derive(CandidType)]
pub enum GetDetailsActorResponse {
    Ok(RegistrationDetails),
    Err(ErrorInfo),
}

impl GetDetailsActorResponse {
    pub fn new(result: ServiceResult<RegistrationDetails>) -> GetDetailsActorResponse {
        match result {
            Ok(details) => GetDetailsActorResponse::Ok(details),
            Err(err) => GetDetailsActorResponse::Err(err.into()),
        }
    }
}

#[query(name = "get_all_details")]
#[candid_method(query)]
pub fn get_all_details(input: GetPageInput) -> GetAllDetailsActorResponse {
    let caller = api::caller();
    let service = RegistrarService::default();
    let result = service.get_all_details(&caller, &input);
    GetAllDetailsActorResponse::new(result)
}

#[derive(CandidType)]
pub enum GetAllDetailsActorResponse {
    Ok(Vec<RegistrationDetails>),
    Err(ErrorInfo),
}

impl GetAllDetailsActorResponse {
    pub fn new(result: ServiceResult<Vec<RegistrationDetails>>) -> GetAllDetailsActorResponse {
        match result {
            Ok(details) => GetAllDetailsActorResponse::Ok(details),
            Err(err) => GetAllDetailsActorResponse::Err(err.into()),
        }
    }
}

#[query(name = "get_last_registrations")]
#[candid_method(query)]
pub fn get_last_registrations() -> GetAllDetailsActorResponse {
    let caller = api::caller();
    let service = RegistrarService::default();
    let result = service.get_last_registrations(&caller);
    GetAllDetailsActorResponse::new(result)
}

#[update(name = "add_quota")]
#[candid_method(update)]
pub fn add_quota(quota_owner: Principal, quota_type: QuotaType, diff: u32) -> BooleanActorResponse {
    if is_env(Production) {
        // it should be always false in production, no one can add quotas in production.
        // all quotas in production should be imported by import_quota.
        BooleanActorResponse::new(Ok(false))
    } else {
        let caller = &api::caller();
        debug!("add_quota: caller: {}", caller);

        let mut service = RegistrarService::default();
        let result = service.add_quota(caller, quota_owner, quota_type, diff);
        BooleanActorResponse::new(result)
    }
}

#[update(name = "import_quota")]
#[candid_method(update)]
pub fn import_quota(request: ImportQuotaRequest) -> ImportQuotaResponse {
    let caller = &api::caller();
    debug!("import_quota: caller: {}", caller);

    let service = RegistrarService::default();
    let result = service.import_quota(caller, request);
    ImportQuotaResponse::new(result)
}

#[derive(CandidType)]
pub enum ImportQuotaResponse {
    Ok(ImportQuotaStatus),
    Err(ErrorInfo),
}

impl ImportQuotaResponse {
    pub fn new(result: ServiceResult<ImportQuotaStatus>) -> ImportQuotaResponse {
        match result {
            Ok(status) => ImportQuotaResponse::Ok(status),
            Err(err) => ImportQuotaResponse::Err(err.into()),
        }
    }
}

#[update(name = "sub_quota")]
#[candid_method(update)]
pub fn sub_quota(quota_owner: Principal, quota_type: QuotaType, diff: u32) -> BooleanActorResponse {
    let caller = &api::caller();
    debug!("sub_quota: caller: {}", caller);

    let mut service = RegistrarService::default();
    let result = service.sub_quota(caller, quota_owner, quota_type, diff);
    BooleanActorResponse::new(result)
}

#[query(name = "get_quota")]
#[candid_method(query)]
pub fn get_quota(quota_owner: Principal, quota_type: QuotaType) -> GetQuotaActorResponse {
    let caller = &api::caller();
    debug!("sub_quota: caller: {}", caller);

    let service = RegistrarService::default();
    let result = service.get_quota(caller, quota_owner, quota_type);
    GetQuotaActorResponse::new(result)
}

#[derive(CandidType)]
pub enum GetQuotaActorResponse {
    Ok(u32),
    Err(ErrorInfo),
}

impl GetQuotaActorResponse {
    pub fn new(result: ServiceResult<u32>) -> GetQuotaActorResponse {
        match result {
            Ok(quota) => GetQuotaActorResponse::Ok(quota),
            Err(err) => GetQuotaActorResponse::Err(err.into()),
        }
    }
}

#[query(name = "get_pending_order")]
#[candid_method(query)]
pub fn get_pending_order() -> GetPendingOrderActorResponse {
    let caller = &api::caller();
    debug!("get_pending_order: caller: {}", caller);

    let service = RegistrarService::default();
    let result = service.get_pending_order(caller);
    GetPendingOrderActorResponse::new(result)
}

#[derive(CandidType)]
pub enum GetPendingOrderActorResponse {
    Ok(Option<GetNameOrderResponse>),
    Err(ErrorInfo),
}

impl GetPendingOrderActorResponse {
    pub fn new(
        result: ServiceResult<Option<GetNameOrderResponse>>,
    ) -> GetPendingOrderActorResponse {
        match result {
            Ok(order) => GetPendingOrderActorResponse::Ok(order),
            Err(err) => GetPendingOrderActorResponse::Err(err.into()),
        }
    }
}

#[update(name = "submit_order")]
#[candid_method(update)]
pub async fn submit_order(request: SubmitOrderRequest) -> SubmitOrderActorResponse {
    let caller = &api::caller();
    debug!("submit_order: caller: {}", caller);
    let now = api::time();

    let service = RegistrarService::default();
    let result = service.submit_order(caller, now, request).await;
    SubmitOrderActorResponse::new(result)
}

#[derive(CandidType)]
pub enum SubmitOrderActorResponse {
    Ok(SubmitOrderResponse),
    Err(ErrorInfo),
}

impl SubmitOrderActorResponse {
    pub fn new(result: ServiceResult<SubmitOrderResponse>) -> SubmitOrderActorResponse {
        match result {
            Ok(order) => SubmitOrderActorResponse::Ok(order),
            Err(err) => SubmitOrderActorResponse::Err(err.into()),
        }
    }
}

#[update(name = "cancel_order")]
#[candid_method(update)]
pub async fn cancel_order() -> BooleanActorResponse {
    let caller = &api::caller();
    debug!("cancel_order: caller: {}", caller);
    let now = api::time();

    let service = RegistrarService::default();
    let result = service.cancel_order(caller, now);
    BooleanActorResponse::new(result)
}

#[update(name = "transfer")]
#[candid_method(update)]
async fn transfer(name: String, new_owner: Principal) -> BooleanActorResponse {
    let caller = &api::caller();

    let service = RegistrarService::default();
    let result = service.transfer(name.as_str(), caller, new_owner).await;
    BooleanActorResponse::new(result)
}

#[update(name = "transfer_by_admin")]
#[candid_method(update)]
async fn transfer_by_admin(name: String, new_owner: Principal) -> BooleanActorResponse {
    let caller = &api::caller();

    let service = RegistrarService::default();
    let result = service
        .transfer_by_admin(name.as_str(), caller, new_owner)
        .await;
    BooleanActorResponse::new(result)
}

#[update(name = "approve")]
#[candid_method(update)]
fn approve(name: String, to: Principal) -> BooleanActorResponse {
    let caller = &api::caller();
    let now = api::time();

    let service = RegistrarService::default();
    let result = service.approve(caller, now, name.as_str(), to);
    BooleanActorResponse::new(result)
}

#[update(name = "transfer_from")]
#[candid_method(update)]
async fn transfer_from(name: String) -> BooleanActorResponse {
    let caller = &api::caller();
    let _now = api::time();

    let service = RegistrarService::default();
    let result = service.transfer_from(caller, name.as_str()).await;
    BooleanActorResponse::new(result)
}

#[update(name = "transfer_quota")]
#[candid_method(update)]
async fn transfer_quota(to: Principal, quota_type: QuotaType, diff: u32) -> BooleanActorResponse {
    let caller = api::caller();

    let service = RegistrarService::default();
    let result = service.transfer_quota(
        caller,
        TransferQuotaDetails {
            to,
            quota_type,
            diff,
        },
    );
    BooleanActorResponse::new(result)
}

#[update(name = "batch_transfer_quota")]
#[candid_method(update)]
async fn batch_transfer_quota(request: BatchTransferRequest) -> BooleanActorResponse {
    let caller = api::caller();

    let service = RegistrarService::default();
    let result = service.batch_transfer_quota(caller, request);
    BooleanActorResponse::new(result)
}

#[derive(CandidType, Deserialize)]
pub struct TransferFromQuotaRequest {
    from: Principal,
    to: Principal,
    quota_type: QuotaType,
    diff: u32,
}

#[update(name = "transfer_from_quota")]
#[candid_method(update)]
async fn transfer_from_quota(request: TransferFromQuotaRequest) -> BooleanActorResponse {
    let caller = &api::caller();

    let service = RegistrarService::default();
    let result = service.transfer_from_quota(
        caller,
        request.from,
        request.to,
        request.quota_type,
        request.diff,
    );
    BooleanActorResponse::new(result)
}

#[update(name = "unlock_names")]
#[candid_method(update)]
async fn unlock_names(names: Vec<String>) -> BooleanActorResponse {
    let caller = &api::caller();

    let service = RegistrarService::default();
    let result = service.unlock_names(caller, names.iter().map(|n| n.as_str()).collect());
    BooleanActorResponse::new(result)
}

#[update(name = "reclaim_name")]
#[candid_method(update)]
async fn reclaim_name(name: String) -> BooleanActorResponse {
    let caller = &api::caller();
    let service = RegistrarService::default();
    let result = service.reclaim_name(name.as_str(), caller).await;
    BooleanActorResponse::new(result)
}

#[query(name = "get_public_resolver")]
#[candid_method(query)]
fn get_public_resolver() -> GetPublicResolverActorResponse {
    let service = RegistrarService::default();
    let result = service.get_public_resolver();
    GetPublicResolverActorResponse::new(Ok(result))
}

#[derive(CandidType)]
pub enum GetPublicResolverActorResponse {
    Ok(String),
    Err(ErrorInfo),
}

impl GetPublicResolverActorResponse {
    pub fn new(result: ServiceResult<String>) -> GetPublicResolverActorResponse {
        match result {
            Ok(data) => GetPublicResolverActorResponse::Ok(data),
            Err(err) => GetPublicResolverActorResponse::Err(err.into()),
        }
    }
}

#[update(name = "renew_name")]
#[candid_method(update)]
async fn renew_name(request: RenewNameRequest) -> BooleanActorResponse {
    let caller = api::caller();
    let now = api::time();
    let service = RegistrarService::default();
    let result = service.renew_name(caller, TimeInNs(now), request).await;
    BooleanActorResponse::new(result)
}

#[update(name = "import_registrations")]
#[candid_method(update)]
async fn import_registrations(request: ImportNameRegistrationRequest) -> BooleanActorResponse {
    let call_context = CallContext::from_ic();
    let service = RegistrarService::default();
    let result = service.import_registrations(call_context, request).await;
    BooleanActorResponse::new(result)
}

candid::export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
#[candid_method(query)]
fn __export_did_tmp_() -> String {
    __export_service()
}

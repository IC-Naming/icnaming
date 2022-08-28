extern crate core;

mod http;
mod name_locker;
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

mod http_nft;
mod token_index_store;

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
use common::nft::{
    AllowanceRequest, ApproveRequest, CommonError, Metadata, NFTServiceResult,
    NFTTransferServiceResult, TransferError, TransferRequest,
};
use common::permissions::must_be_named_principal;
use common::token_identifier::TokenIdentifier;
use common::{CallContext, TimeInNs};

use crate::periodic_tasks_runner::run_periodic_tasks;
use crate::registration_store::{RegistrationDetails, RegistrationDto};
use crate::service::*;

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

#[update(name = "register_with_payment")]
#[candid_method(update)]
pub async fn register_with_payment(
    request: RegisterNameWithPaymentRequest,
) -> GetDetailsActorResponse {
    let call_context = CallContext::from_ic();
    let service = RegistrarService::default();
    let result = service.register_with_payment(call_context, request).await;
    GetDetailsActorResponse::new(result)
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

#[query(name = "get_names_count")]
#[candid_method(query)]
pub fn get_names_count(owner: Principal) -> GetNamesCountActorResponse {
    let service = RegistrarService::default();
    let result = service.get_names_count(&owner);
    GetNamesCountActorResponse::new(result)
}

#[derive(CandidType)]
pub enum GetNamesCountActorResponse {
    Ok(u32),
    Err(ErrorInfo),
}

impl GetNamesCountActorResponse {
    pub fn new(result: ServiceResult<u32>) -> GetNamesCountActorResponse {
        match result {
            Ok(count) => GetNamesCountActorResponse::Ok(count),
            Err(err) => GetNamesCountActorResponse::Err(err.into()),
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
    let call_context = CallContext::from_ic();
    let service = RegistrarService::default();
    let result = service.get_all_details(call_context, &input);
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

#[update(name = "batch_add_quota")]
#[candid_method(update)]
pub fn batch_add_quota(request: BatchAddQuotaRequest) -> BooleanActorResponse {
    let mut service = RegistrarService::default();
    let context = CallContext::from_ic();
    let result = service.batch_add_quota(context, request);
    BooleanActorResponse::new(result)
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
    let result = service.transfer_from(caller, name.as_str(), None).await;
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

#[query(name = "get_name_status")]
#[candid_method(query)]
fn get_name_status(name: String) -> GetNameStatueActorResponse {
    let service = RegistrarService::default();
    let result = service.get_name_status(name.as_str());
    GetNameStatueActorResponse::new(result)
}

#[derive(CandidType)]
pub enum GetNameStatueActorResponse {
    Ok(NameStatus),
    Err(ErrorInfo),
}

impl GetNameStatueActorResponse {
    pub fn new(result: ServiceResult<NameStatus>) -> GetNameStatueActorResponse {
        match result {
            Ok(data) => GetNameStatueActorResponse::Ok(data),
            Err(err) => GetNameStatueActorResponse::Err(err.into()),
        }
    }
}

pub type GetRegistryActorResponse = Vec<(u32, String)>;

#[query(name = "getRegistry")]
#[candid_method(query, rename = "getRegistry")]
pub fn get_registry() -> GetRegistryActorResponse {
    let service = RegistrarService::default();
    let now = api::time();
    let result = service.get_registry(now);
    result
}

pub type GetTokens = Vec<(u32, Metadata)>;

#[query(name = "getTokens")]
#[candid_method(query, rename = "getTokens")]
pub fn get_tokens() -> GetTokens {
    let service = RegistrarService::default();
    let now = api::time();
    let result = service.get_tokens(now);
    result
}

#[derive(CandidType)]
pub enum MetadataActorResponse {
    Ok(Metadata),
    Err(CommonError),
}

impl MetadataActorResponse {
    pub fn new(result: NFTServiceResult<Metadata>) -> MetadataActorResponse {
        match result {
            Ok(data) => MetadataActorResponse::Ok(data),
            Err(err) => MetadataActorResponse::Err(err.into()),
        }
    }
}

#[query(name = "metadata")]
#[candid_method(query)]
pub fn metadata(token: TokenIdentifier) -> MetadataActorResponse {
    let service = RegistrarService::default();
    let now = api::time();
    let result = service.metadata(&token, now);
    MetadataActorResponse::new(result)
}

#[derive(CandidType)]
pub enum SupplyActorResponse {
    Ok(u128),
    Err(CommonError),
}

impl SupplyActorResponse {
    pub fn new(result: NFTServiceResult<u128>) -> SupplyActorResponse {
        match result {
            Ok(data) => SupplyActorResponse::Ok(data),
            Err(err) => SupplyActorResponse::Err(err.into()),
        }
    }
}

#[query(name = "supply")]
#[candid_method(query)]
pub fn supply() -> SupplyActorResponse {
    let service = RegistrarService::default();
    let result = service.supply();
    SupplyActorResponse::new(result)
}

pub type GetMinterActorResponse = Principal;

#[query(name = "getMinter")]
#[candid_method(query, rename = "getMinter")]
pub fn minter() -> GetMinterActorResponse {
    Principal::anonymous()
}

#[derive(CandidType)]
pub enum BearerActorResponse {
    Ok(String),
    Err(CommonError),
}

impl BearerActorResponse {
    pub fn new(result: NFTServiceResult<String>) -> BearerActorResponse {
        match result {
            Ok(data) => BearerActorResponse::Ok(data),
            Err(err) => BearerActorResponse::Err(err.into()),
        }
    }
}

#[query(name = "bearer")]
#[candid_method(query)]
pub fn bearer(token: TokenIdentifier) -> BearerActorResponse {
    let service = RegistrarService::default();
    let now = api::time();
    let result = service.bearer(&token, now);
    BearerActorResponse::new(result)
}

#[derive(CandidType)]
pub enum ImportTokenIdResponse {
    Ok(usize),
    Err(ErrorInfo),
}

impl ImportTokenIdResponse {
    pub fn new(result: ServiceResult<usize>) -> ImportTokenIdResponse {
        match result {
            Ok(data) => ImportTokenIdResponse::Ok(data),
            Err(err) => ImportTokenIdResponse::Err(err.into()),
        }
    }
}

#[update(name = "import_token_id_from_registration")]
#[candid_method(update)]
pub fn import_token_id_from_registration() -> ImportTokenIdResponse {
    let service = RegistrarService::default();
    let call_context = CallContext::from_ic();
    let result = service.import_token_id_from_registration(&call_context);
    ImportTokenIdResponse::new(result)
}

#[update(name = "ext_approve")]
#[candid_method(update)]
pub fn ext_approve(request: ApproveRequest) {
    let service = RegistrarService::default();
    let call_context = CallContext::from_ic();
    let now = api::time();
    let _ = service.ext_approve(&call_context, request.spender, &request.token, now);
}

#[derive(CandidType)]
pub enum AllowanceActorResponse {
    Ok(u128),
    Err(CommonError),
}

impl AllowanceActorResponse {
    pub fn new(result: NFTServiceResult<u128>) -> AllowanceActorResponse {
        match result {
            Ok(data) => AllowanceActorResponse::Ok(data),
            Err(err) => AllowanceActorResponse::Err(err.into()),
        }
    }
}

#[query(name = "allowance")]
#[candid_method(query)]
pub fn allowance(request: AllowanceRequest) -> AllowanceActorResponse {
    let service = RegistrarService::default();
    let now = api::time();
    let result = service.allowance(&request.owner, &request.spender, &request.token, now);
    AllowanceActorResponse::new(result)
}

#[derive(CandidType)]
pub enum EXTTransferResponse {
    Ok(u128),
    Err(TransferError),
}

impl EXTTransferResponse {
    pub fn new(result: NFTTransferServiceResult<u128>) -> EXTTransferResponse {
        match result {
            Ok(data) => EXTTransferResponse::Ok(data),
            Err(err) => EXTTransferResponse::Err(err.into()),
        }
    }
}

#[update(name = "ext_transfer")]
#[candid_method(update)]
pub async fn ext_transfer(request: TransferRequest) -> EXTTransferResponse {
    let service = RegistrarService::default();
    let call_context = CallContext::from_ic();
    let now = api::time();
    let result = service
        .ext_transfer(
            &call_context,
            &request.from,
            &request.to,
            &request.token,
            now,
        )
        .await;
    EXTTransferResponse::new(result)
}

pub type GetTokenIdListByNamesResponse = HashMap<String, Option<(u32, String)>>;

#[query(name = "get_token_details_by_names")]
#[candid_method(query)]
pub fn get_token_details_by_names(names: Vec<String>) -> GetTokenIdListByNamesResponse {
    let service = RegistrarService::default();
    let now = api::time();
    let result = service.get_token_details_by_names(&names, now);
    result
}

candid::export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
#[candid_method(query)]
fn __export_did_tmp_() -> String {
    __export_service()
}

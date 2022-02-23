use std::collections::HashSet;

use candid::{candid_method, CandidType, Deserialize, Principal};
use ic_cdk::api;
use ic_cdk_macros::*;
use log::debug;

use common::dto::{to_state_export_data, GetPageInput, GetPageOutput, StateExportResponse};
use common::errors::{BooleanActorResponse, ErrorInfo, ICNSActorResult, ICNSResult};
use common::permissions::must_be_system_owner;
use common::state::StableState;

use crate::astrox_me_name::ImportedStats;
use crate::name_order_store::GetNameOrderResponse;
use crate::registration_store::{RegistrationDetails, RegistrationDto};
use crate::service::{
    PriceTable, RegistrarService, Stats, SubmitOrderRequest, SubmitOrderResponse,
};
use crate::state::{State, STATE};
use crate::user_quota_store::QuotaType;

#[query(name = "get_stats")]
#[candid_method(query, rename = "get_stats")]
pub fn get_stats() -> GetStatsActorResponse {
    let now = api::time();
    let service = RegistrarService::new();
    let stats = service.get_stats(now);
    GetStatsActorResponse::new(Ok(stats))
}

#[derive(CandidType)]
pub enum GetStatsActorResponse {
    Ok(Stats),
    Err(ErrorInfo),
}

#[update(name = "export_state")]
#[candid_method(update, rename = "export_state")]
pub async fn export_state() -> StateExportResponse {
    let caller = &api::caller();
    let permission_result = must_be_system_owner(caller);
    if permission_result.is_err() {
        return StateExportResponse::new(Err(permission_result.err().unwrap()));
    }

    let source_data = STATE.with(|state| to_state_export_data(state.encode()));
    StateExportResponse::new(Ok(source_data))
}

impl GetStatsActorResponse {
    pub fn new(result: ICNSResult<Stats>) -> GetStatsActorResponse {
        match result {
            Ok(stats) => GetStatsActorResponse::Ok(stats),
            Err(err) => GetStatsActorResponse::Err(err.into()),
        }
    }
}

#[update(name = "get_price_table")]
#[candid_method(update, rename = "get_price_table")]
pub async fn get_price_table() -> GetPriceTableResponse {
    let service = RegistrarService::new();
    let price_table = service.get_price_table().await;
    GetPriceTableResponse::new(price_table)
}

#[derive(CandidType)]
pub enum GetPriceTableResponse {
    Ok(PriceTable),
    Err(ErrorInfo),
}

impl GetPriceTableResponse {
    pub fn new(result: ICNSResult<PriceTable>) -> GetPriceTableResponse {
        match result {
            Ok(price_table) => GetPriceTableResponse::Ok(price_table),
            Err(err) => GetPriceTableResponse::Err(err.into()),
        }
    }
}

/// Check if name is available.
/// Returns true if name is available.
///
/// * `name` - name to check, e.g. "hello.icp"
#[query(name = "available")]
#[candid_method(query, rename = "available")]
pub fn available(name: String) -> BooleanActorResponse {
    let service = RegistrarService::new();
    match service.available(&name) {
        Ok(result) => BooleanActorResponse::new(Ok(true)),
        Err(err) => BooleanActorResponse::new(Err(err.into())),
    }
}

/// Get expiration date for a name.
/// Returns expiration date.
///
/// * `name` - name to get, e.g. "hello.icp"
#[query(name = "get_name_expires")]
#[candid_method(query, rename = "get_name_expires")]
pub fn get_name_expires(name: String) -> GetNameExpiresActorResponse {
    let service = RegistrarService::new();
    let result = service.get_name_expires(&name);
    GetNameExpiresActorResponse::new(result)
}
#[derive(CandidType)]
pub enum GetNameExpiresActorResponse {
    Ok(u64),
    Err(ErrorInfo),
}

impl GetNameExpiresActorResponse {
    pub fn new(result: ICNSResult<u64>) -> GetNameExpiresActorResponse {
        match result {
            Ok(expires) => GetNameExpiresActorResponse::Ok(expires),
            Err(err) => GetNameExpiresActorResponse::Err(err.into()),
        }
    }
}

/// Register a name for a owner. This is private method for activity client.
/// Returns true if name is registered successfully.
///
/// * `name` - name to register, e.g. "hello.icp"
/// * `owner` - owner to register the name for
/// * `years` - number of years to register the name for
#[update(name = "register_for")]
#[candid_method(update, rename = "register_for")]
pub async fn register_for(name: String, owner: Principal, years: u64) -> BooleanActorResponse {
    let caller = &api::caller();
    debug!("register_for: caller: {}", caller);

    let mut service = RegistrarService::new();
    let result = service
        .register(
            &name,
            &owner,
            years as u32,
            api::time(),
            caller,
            QuotaType::LenGte(4),
        )
        .await;
    BooleanActorResponse::new(result)
}

/// Register a name for a caller with a quota.
/// Returns true if name is registered successfully.
///
/// * `name` - name to register, e.g. "hello.icp"
/// * `quota_type` - quota type to use
#[update(name = "register_with_quota")]
#[candid_method(update, rename = "register_with_quota")]
pub async fn register_with_quota(name: String, quota_type: QuotaType) -> BooleanActorResponse {
    let caller = &api::caller();
    debug!("register_with_quota: caller: {}", caller);

    let mut service = RegistrarService::new();
    let years = 1;
    let result = service
        .register(&name, &caller, years, api::time(), &caller, quota_type)
        .await;
    BooleanActorResponse::new(result)
}

/// Get names for a owner.
/// Returns names for a owner.
///
/// * `owner` - owner to get names for
/// * `page` - page offset and limit
#[query(name = "get_names")]
#[candid_method(query, rename = "get_names")]
pub fn get_names(owner: Principal, input: GetPageInput) -> GetNamesActorResponse {
    let service = RegistrarService::new();
    let result = service.get_names(&owner, &input);
    GetNamesActorResponse::new(result)
}

#[derive(CandidType)]
pub enum GetNamesActorResponse {
    Ok(GetPageOutput<RegistrationDto>),
    Err(ErrorInfo),
}

impl GetNamesActorResponse {
    pub fn new(result: ICNSResult<GetPageOutput<RegistrationDto>>) -> GetNamesActorResponse {
        match result {
            Ok(output) => GetNamesActorResponse::Ok(output),
            Err(err) => GetNamesActorResponse::Err(err.into()),
        }
    }
}

/// get owner for a name.
/// Returns owner for a name.
///
/// * `name` - name to get owner for
#[query(name = "get_owner")]
#[candid_method(query, rename = "get_owner")]
pub fn get_owner(name: String) -> GetOwnerActorResponse {
    let service = RegistrarService::new();
    let result = service.get_owner(&name);
    GetOwnerActorResponse::new(result)
}

#[derive(CandidType)]
pub enum GetOwnerActorResponse {
    Ok(Principal),
    Err(ErrorInfo),
}

impl GetOwnerActorResponse {
    pub fn new(result: ICNSResult<Principal>) -> GetOwnerActorResponse {
        match result {
            Ok(owner) => GetOwnerActorResponse::Ok(owner),
            Err(err) => GetOwnerActorResponse::Err(err.into()),
        }
    }
}

/// Get details for a name.
/// Returns details for a name.
///
/// * `name` - name to get details for
#[query(name = "get_details")]
#[candid_method(query, rename = "get_details")]
pub fn get_details(name: String) -> GetDetailsActorResponse {
    let service = RegistrarService::new();
    let result = service.get_details(&name);
    GetDetailsActorResponse::new(result)
}

#[derive(CandidType)]
pub enum GetDetailsActorResponse {
    Ok(RegistrationDetails),
    Err(ErrorInfo),
}

impl GetDetailsActorResponse {
    pub fn new(result: ICNSResult<RegistrationDetails>) -> GetDetailsActorResponse {
        match result {
            Ok(details) => GetDetailsActorResponse::Ok(details),
            Err(err) => GetDetailsActorResponse::Err(err.into()),
        }
    }
}

/// Get all details
/// Returns all name details.
///
/// * `name` - name to get details for
#[query(name = "get_all_details")]
#[candid_method(query, rename = "get_all_details")]
pub fn get_all_details(input: GetPageInput) -> GetAllDetailsActorResponse {
    let caller = api::caller();
    let service = RegistrarService::new();
    let result = service.get_all_details(&caller, &input);
    GetAllDetailsActorResponse::new(result)
}

#[derive(CandidType)]
pub enum GetAllDetailsActorResponse {
    Ok(Vec<RegistrationDetails>),
    Err(ErrorInfo),
}

impl GetAllDetailsActorResponse {
    pub fn new(result: ICNSResult<Vec<RegistrationDetails>>) -> GetAllDetailsActorResponse {
        match result {
            Ok(details) => GetAllDetailsActorResponse::Ok(details),
            Err(err) => GetAllDetailsActorResponse::Err(err.into()),
        }
    }
}

/// Add quotas to a quota owner.
/// Returns true if quotas are added successfully.
///
/// * `quota_owner` - owner to add quotas to
/// * `quota_type` - quota type to add
/// * `diff` - number of quotas to add
#[update(name = "add_quota")]
#[candid_method(update, rename = "add_quota")]
pub fn add_quota(quota_owner: Principal, quota_type: QuotaType, diff: u32) -> BooleanActorResponse {
    let caller = &api::caller();
    debug!("add_quota: caller: {}", caller);

    let mut service = RegistrarService::new();
    let result = service.add_quota(caller, quota_owner, quota_type, diff);
    BooleanActorResponse::new(result)
}

/// Remove quotas from a quota owner.
/// Returns true if quotas are removed successfully.
///
/// * `quota_owner` - owner to remove quotas from
/// * `quota_type` - quota type to remove
/// * `diff` - number of quotas to remove
#[update(name = "sub_quota")]
#[candid_method(update, rename = "sub_quota")]
pub fn sub_quota(quota_owner: Principal, quota_type: QuotaType, diff: u32) -> BooleanActorResponse {
    let caller = &api::caller();
    debug!("sub_quota: caller: {}", caller);

    let mut service = RegistrarService::new();
    let result = service.sub_quota(caller, quota_owner, quota_type, diff);
    BooleanActorResponse::new(result)
}

/// Get quotas for a quota owner.
/// Returns quotas for a quota owner.
///
/// * `quota_owner` - owner to get quotas for
/// * `quota_type` - quota type to get
#[query(name = "get_quota")]
#[candid_method(query, rename = "get_quota")]
pub fn get_quota(quota_owner: Principal, quota_type: QuotaType) -> GetQuotaActorResponse {
    let caller = &api::caller();
    debug!("sub_quota: caller: {}", caller);

    let service = RegistrarService::new();
    let result = service.get_quota(caller, quota_owner, quota_type);
    GetQuotaActorResponse::new(result)
}

#[derive(CandidType)]
pub enum GetQuotaActorResponse {
    Ok(u32),
    Err(ErrorInfo),
}
impl GetQuotaActorResponse {
    pub fn new(result: ICNSResult<u32>) -> GetQuotaActorResponse {
        match result {
            Ok(quota) => GetQuotaActorResponse::Ok(quota),
            Err(err) => GetQuotaActorResponse::Err(err.into()),
        }
    }
}

#[query(name = "get_pending_order")]
#[candid_method(query, rename = "get_pending_order")]
pub fn get_pending_order() -> GetPendingOrderActorResponse {
    let caller = &api::caller();
    debug!("get_pending_order: caller: {}", caller);

    let service = RegistrarService::new();
    let result = service.get_pending_order(caller);
    GetPendingOrderActorResponse::new(result)
}

#[derive(CandidType)]
pub enum GetPendingOrderActorResponse {
    Ok(Option<GetNameOrderResponse>),
    Err(ErrorInfo),
}

impl GetPendingOrderActorResponse {
    pub fn new(result: ICNSResult<Option<GetNameOrderResponse>>) -> GetPendingOrderActorResponse {
        match result {
            Ok(order) => GetPendingOrderActorResponse::Ok(order),
            Err(err) => GetPendingOrderActorResponse::Err(err.into()),
        }
    }
}

#[update(name = "submit_order")]
#[candid_method(update, rename = "submit_order")]
pub async fn submit_order(request: SubmitOrderRequest) -> SubmitOrderActorResponse {
    let caller = &api::caller();
    debug!("submit_order: caller: {}", caller);
    let now = api::time();

    let service = RegistrarService::new();
    let result = service.submit_order(caller, now, request).await;
    SubmitOrderActorResponse::new(result)
}

#[derive(CandidType)]
pub enum SubmitOrderActorResponse {
    Ok(SubmitOrderResponse),
    Err(ErrorInfo),
}

impl SubmitOrderActorResponse {
    pub fn new(result: ICNSResult<SubmitOrderResponse>) -> SubmitOrderActorResponse {
        match result {
            Ok(order) => SubmitOrderActorResponse::Ok(order),
            Err(err) => SubmitOrderActorResponse::Err(err.into()),
        }
    }
}

#[update(name = "cancel_order")]
#[candid_method(update, rename = "cancel_order")]
pub async fn cancel_order() -> BooleanActorResponse {
    let caller = &api::caller();
    debug!("cancel_order: caller: {}", caller);
    let now = api::time();

    let service = RegistrarService::new();
    let result = service.cancel_order(caller, now);
    BooleanActorResponse::new(result)
}

#[update(name = "refund_order")]
#[candid_method(update, rename = "refund_order")]
pub async fn refund_order() -> BooleanActorResponse {
    let caller = &api::caller();
    debug!("refund_order: caller: {}", caller);
    let now = api::time();

    let service = RegistrarService::new();
    let result = service.refund_order(caller, now).await;
    BooleanActorResponse::new(result)
}

#[query(name = "get_astrox_me_name_stats")]
#[candid_method(query, rename = "get_astrox_me_name_stats")]
pub fn get_astrox_me_name_stats() -> GetAstroxMeNameStatsActorResponse {
    let service = RegistrarService::new();
    let result = service.get_astrox_me_name_stats();
    GetAstroxMeNameStatsActorResponse::new(result)
}

#[derive(CandidType)]
pub enum GetAstroxMeNameStatsActorResponse {
    Ok(ImportedStats),
    Err(ErrorInfo),
}

impl GetAstroxMeNameStatsActorResponse {
    pub fn new(result: ICNSResult<ImportedStats>) -> GetAstroxMeNameStatsActorResponse {
        match result {
            Ok(stats) => GetAstroxMeNameStatsActorResponse::Ok(stats),
            Err(err) => GetAstroxMeNameStatsActorResponse::Err(err.into()),
        }
    }
}

#[update(name = "import_astrox_me_names")]
#[candid_method(update, rename = "import_astrox_me_names")]
pub async fn import_astrox_me_names(
    mut names: HashSet<String>,
) -> ImportAstroxMeNamesActorResponse {
    let caller = &api::caller();
    debug!("import_astrox_me_names: caller: {}", caller);
    let now = api::time();

    let mut service = RegistrarService::new();
    let result = service.import_astrox_me_names(caller, now, names).await;
    ImportAstroxMeNamesActorResponse::new(result)
}

#[derive(CandidType)]
pub enum ImportAstroxMeNamesActorResponse {
    Ok(ImportedStats),
    Err(ErrorInfo),
}

impl ImportAstroxMeNamesActorResponse {
    pub fn new(result: ICNSResult<ImportedStats>) -> ImportAstroxMeNamesActorResponse {
        match result {
            Ok(stats) => ImportAstroxMeNamesActorResponse::Ok(stats),
            Err(err) => ImportAstroxMeNamesActorResponse::Err(err.into()),
        }
    }
}

candid::export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
#[candid_method(query, rename = "__get_candid_interface_tmp_hack")]
fn __export_did_tmp_() -> String {
    __export_service()
}

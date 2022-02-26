use std::fmt::Debug;

use async_trait::async_trait;
use candid::{CandidType, Principal};
use ic_cdk::api::call::RejectionCode;
use ic_cdk::call;
use log::{debug, error};
use serde::Deserialize;

use crate::cycles_minting_types::IcpXdrConversionRateCertifiedResponse;
use crate::dto::*;
use crate::errors::{ErrorInfo, ICNSActorResult, ICNSError};
use crate::icnaming_ledger_types::*;
use crate::named_canister_ids::get_named_get_canister_id;

pub mod ic_impl;

async fn call_core<T, TResult>(
    canister_name: &str,
    method: &str,
    args: T,
    logging: bool,
) -> Result<TResult, ICNSError>
where
    T: candid::utils::ArgumentEncoder,
    TResult: for<'a> Deserialize<'a> + CandidType + Debug,
{
    if logging {
        debug!("Calling {}::{}", canister_name, method);
    }
    let canister_id = get_named_get_canister_id(canister_name);
    let call_result: Result<(TResult,), (RejectionCode, String)> =
        call(canister_id, method, args).await;
    if call_result.is_err() {
        let (code, message) = call_result.err().unwrap();
        let code_string = format!("{:?}", code);
        error!(
            "{}::{} failed with code {}: {}",
            canister_name, method, code_string, message
        );
        return Err(ICNSError::CanisterCallError {
            message,
            rejection_code: code_string,
        });
    }
    let result = call_result.unwrap();
    if logging {
        debug!(
            "Call canister {} with method {} result: {:?}",
            canister_name, method, result
        );
    }
    Ok(result.0)
}

async fn call_canister_as_icns_result<T, TResult>(
    canister_name: &str,
    method: &str,
    args: T,
) -> ICNSActorResult<TResult>
where
    T: candid::utils::ArgumentEncoder,
    TResult: for<'a> Deserialize<'a> + CandidType + Debug,
{
    call_core::<T, ICNSActorResult<TResult>>(canister_name, method, args, true)
        .await
        .map(|result| result.unwrap())
        .map_err(ErrorInfo::from)
}

async fn call_canister_as_result<T, TResult>(
    canister_name: &str,
    method: &str,
    args: T,
) -> ICNSActorResult<TResult>
where
    T: candid::utils::ArgumentEncoder,
    TResult: for<'a> Deserialize<'a> + CandidType + Debug,
{
    call_core::<T, TResult>(canister_name, method, args, true)
        .await
        .map_err(ErrorInfo::from)
}

async fn call_canister_as_result_no_logging<T, TResult>(
    canister_name: &str,
    method: &str,
    args: T,
) -> ICNSActorResult<TResult>
where
    T: candid::utils::ArgumentEncoder,
    TResult: for<'a> Deserialize<'a> + CandidType + Debug,
{
    call_core::<T, TResult>(canister_name, method, args, false)
        .await
        .map_err(ErrorInfo::from)
}

#[async_trait]
pub trait IRegistryApi {
    async fn set_subdomain_owner(
        &self,
        label: String,
        parent_name: String,
        sub_owner: Principal,
        ttl: u64,
        resolver: Principal,
    ) -> ICNSActorResult<RegistryDto>;

    async fn get_resolver(&self, label: &str) -> ICNSActorResult<Principal>;
    async fn get_users(&self, name: &str) -> ICNSActorResult<RegistryUsers>;
}

#[async_trait]
pub trait IResolverApi {
    async fn ensure_resolver_created(&self, name: String) -> ICNSActorResult<bool>;
}

#[async_trait]
pub trait IICNamingLedgerApi {
    async fn add_payment(&self, request: AddPaymentRequest) -> ICNSActorResult<AddPaymentResponse>;
    async fn verify_payment(
        &self,
        request: VerifyPaymentRequest,
    ) -> ICNSActorResult<VerifyPaymentResponse>;
    async fn get_tip_of_ledger(
        &self,
        request: GetTipOfLedgerRequest,
    ) -> ICNSActorResult<GetTipOfLedgerResponse>;
    async fn refund_payment(
        &self,
        request: RefundPaymentRequest,
    ) -> ICNSActorResult<RefundPaymentResponse>;
    async fn sync_icp_payment(
        &self,
        request: SyncICPPaymentRequest,
    ) -> ICNSActorResult<VerifyPaymentResponse>;
}

#[async_trait]
pub trait ICyclesMintingApi {
    async fn get_icp_xdr_conversion_rate(
        &self,
    ) -> ICNSActorResult<IcpXdrConversionRateCertifiedResponse>;
}

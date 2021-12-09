use std::fmt::Debug;

use async_trait::async_trait;
use candid::{CandidType, Principal};
use ic_cdk::api;
use log::debug;
use serde::Deserialize;

use crate::constants::CANISTER_NAME_RESOLVER;
use crate::dto::*;
use crate::errors::ICNSActorResult;
use crate::state::get_principal;

pub mod ic_impl;

async fn call_canister<T, TResult>(
    canister_name: &str,
    method: &str,
    args: T,
) -> ICNSActorResult<TResult>
where
    T: candid::utils::ArgumentEncoder,
    TResult: for<'a> Deserialize<'a> + CandidType + Debug,
{
    debug!("Calling canister {} with method {}", canister_name, method);
    let canister_name = get_principal(canister_name).unwrap();
    let call1: Result<(ICNSActorResult<TResult>,), _> =
        api::call::call(canister_name.clone(), method, args).await;
    let result = call1.unwrap().0;
    debug!(
        "Call canister {} with method {} result: {:?}",
        canister_name, method, result
    );
    result
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

use async_trait::async_trait;
use candid::{Nat, Principal};
use mockall::{mock, predicate::*};
use rstest::*;

use common::canister_api::*;
use common::cycles_minting_types::*;
use common::dto::*;
use common::errors::ActorResult;

mock! {
    pub RegistryApi {
    }
    #[async_trait]
impl IRegistryApi for RegistryApi {
    async fn set_subdomain_owner(
        &self,
        label: String,
        parent_name: String,
        sub_owner: Principal,
        ttl: u64,
        resolver: Principal,
    ) -> ActorResult<RegistryDto>;

    async fn reclaim_name(
        &self,
        name: String,
        owner: Principal,
        resolver: Principal,
    ) -> ActorResult<bool>;
    async fn transfer(
        &self,
        name: String,
        new_owner: Principal,
        resolver: Principal,
    ) -> ActorResult<bool>;
    async fn get_resolver(&self, label: &str) -> ActorResult<Principal>;
    async fn get_users(&self, name: &str) -> ActorResult<RegistryUsers>;
}
}

#[fixture]
pub fn mock_registry_api() -> MockRegistryApi {
    MockRegistryApi::new()
}

mock! {
    pub ResolverApi {
    }
    #[async_trait]
impl IResolverApi for ResolverApi {
    async fn ensure_resolver_created(&self, name: String) -> ActorResult<bool>;
    async fn remove_resolvers(&self, names: Vec<String>) -> ActorResult<bool>;
}
}

#[fixture]
pub fn mock_resolver_api() -> MockResolverApi {
    MockResolverApi::new()
}

mock! {
    pub CyclesMintingApi {
    }
    #[async_trait]
impl ICyclesMintingApi for CyclesMintingApi {
    async fn get_icp_xdr_conversion_rate(&self) -> ActorResult<IcpXdrConversionRateCertifiedResponse>;
}
}

#[fixture]
pub fn mock_cycles_minting_api() -> MockCyclesMintingApi {
    MockCyclesMintingApi::new()
}

mock! {
    pub RegistrarApi {
    }
    #[async_trait]
impl IRegistrarApi for RegistrarApi {
    async fn import_quota(&self, request: ImportQuotaRequest)
        -> ActorResult<ImportQuotaStatus>;
    async fn register_from_gateway(&self, name: String, owner: Principal) -> ActorResult<bool>;
}
}

#[fixture]
pub fn mock_registrar_api() -> MockRegistrarApi {
    MockRegistrarApi::new()
}

mock! {
    pub DICPApi {
    }
    #[async_trait]
impl IDICPApi for DICPApi {
    async fn transfer_from(
        &self,
        spender_sub_account: Option<Subaccount>,
        from: String,
        to: String,
        value: Nat,
        created_at: Option<u64>,
    ) -> ActorResult<TransactionResponse>;

    async fn transfer(
        &self,
        from_sub_account: Option<Subaccount>,
        to: String,
        value: Nat,
        created_at: Option<u64>,
    ) -> ActorResult<TransactionResponse>;

    async fn balance_of(&self, token_holder: String) -> ActorResult<Nat>;
}
}

#[fixture]
pub fn mock_dicp_api() -> MockDICPApi {
    MockDICPApi::new()
}

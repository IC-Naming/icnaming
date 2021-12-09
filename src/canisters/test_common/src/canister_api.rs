use async_trait::async_trait;
use candid::Principal;
use mockall::{mock, predicate::*};
use rstest::*;

use common::canister_api::*;
use common::dto::*;
use common::errors::ICNSActorResult;

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
    ) -> ICNSActorResult<RegistryDto>;

    async fn get_resolver(&self, label: &str) -> ICNSActorResult<Principal>;
    async fn get_users(&self, name: &str) -> ICNSActorResult<RegistryUsers>;
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
    async fn ensure_resolver_created(&self, name: String) -> ICNSActorResult<bool>;
}
}

#[fixture]
pub fn mock_resolver_api() -> MockResolverApi {
    MockResolverApi::new()
}

use candid::Principal;

use crate::constants::CANISTER_NAME_REGISTRY;
use crate::errors::ICNSActorResult;

use super::*;

pub struct RegistryApi;

impl RegistryApi {
    pub fn new() -> RegistryApi {
        Self
    }
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
    ) -> ICNSActorResult<RegistryDto> {
        call_canister(
            CANISTER_NAME_REGISTRY,
            "set_subdomain_owner",
            (label, parent_name, sub_owner, ttl, resolver),
        )
        .await
    }

    async fn get_resolver(&self, label: &str) -> ICNSActorResult<Principal> {
        call_canister(CANISTER_NAME_REGISTRY, "get_resolver", (label,)).await
    }

    async fn get_users(&self, name: &str) -> ICNSActorResult<RegistryUsers> {
        call_canister(CANISTER_NAME_REGISTRY, "get_users", (name,)).await
    }
}

pub struct ResolverApi;

impl ResolverApi {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl IResolverApi for ResolverApi {
    async fn ensure_resolver_created(&self, name: String) -> ICNSActorResult<bool> {
        call_canister(CANISTER_NAME_RESOLVER, "ensure_resolver_created", (name,)).await
    }
}

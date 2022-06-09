use super::*;
use crate::named_canister_ids::CanisterNames;

#[derive(Default)]
pub struct RegistrarApi;

#[async_trait]
impl IRegistrarApi for RegistrarApi {
    async fn import_quota(&self, request: ImportQuotaRequest) -> ActorResult<ImportQuotaStatus> {
        call_canister_as_icns_result(CanisterNames::Registrar, "import_quota", (request,)).await
    }

    async fn register_from_gateway(&self, name: String, owner: Principal) -> ActorResult<bool> {
        call_canister_as_icns_result(
            CanisterNames::Registrar,
            "register_from_gateway",
            (name, owner),
        )
        .await
    }
}

#[derive(Default)]
pub struct RegistryApi;

#[async_trait]
impl IRegistryApi for RegistryApi {
    async fn set_subdomain_owner(
        &self,
        label: String,
        parent_name: String,
        sub_owner: Principal,
        ttl: u64,
        resolver: Principal,
    ) -> ActorResult<RegistryDto> {
        call_canister_as_icns_result(
            CanisterNames::Registry,
            "set_subdomain_owner",
            (label, parent_name, sub_owner, ttl, resolver),
        )
        .await
    }

    async fn reclaim_name(
        &self,
        name: String,
        owner: Principal,
        resolver: Principal,
    ) -> ActorResult<bool> {
        call_canister_as_icns_result(
            CanisterNames::Registry,
            "reclaim_name",
            (name, owner, resolver),
        )
        .await
    }

    async fn transfer(
        &self,
        name: String,
        new_owner: Principal,
        resolver: Principal,
    ) -> ActorResult<bool> {
        call_canister_as_icns_result(
            CanisterNames::Registry,
            "transfer",
            (name, new_owner, resolver),
        )
        .await
    }

    async fn get_resolver(&self, label: &str) -> ActorResult<Principal> {
        call_canister_as_icns_result(CanisterNames::Registry, "get_resolver", (label,)).await
    }

    async fn get_users(&self, name: &str) -> ActorResult<RegistryUsers> {
        call_canister_as_icns_result(CanisterNames::Registry, "get_users", (name,)).await
    }
}

#[derive(Default)]
pub struct ResolverApi;

#[async_trait]
impl IResolverApi for ResolverApi {
    async fn ensure_resolver_created(&self, name: String) -> ActorResult<bool> {
        call_canister_as_icns_result(CanisterNames::Resolver, "ensure_resolver_created", (name,))
            .await
    }

    async fn remove_resolvers(&self, names: Vec<String>) -> ActorResult<bool> {
        call_canister_as_icns_result(CanisterNames::Resolver, "remove_resolvers", (names,)).await
    }
}

#[derive(Default)]
pub struct CyclesMintingApi;

impl CyclesMintingApi {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ICyclesMintingApi for CyclesMintingApi {
    async fn get_icp_xdr_conversion_rate(
        &self,
    ) -> ActorResult<IcpXdrConversionRateCertifiedResponse> {
        call_canister_as_result_no_logging(
            CanisterNames::CyclesMinting,
            "get_icp_xdr_conversion_rate",
            (),
        )
        .await
    }
}

#[derive(Debug, Default)]
pub struct DICPApi {}

#[async_trait]
impl IDICPApi for DICPApi {
    async fn transfer_from(
        &self,
        spender_sub_account: Option<Subaccount>,
        from: String,
        to: String,
        value: Nat,
        nonce: Option<u64>,
    ) -> ActorResult<TransactionResponse> {
        call_canister_as_icns_result(
            CanisterNames::DICP,
            "transferFrom",
            (spender_sub_account, from, to, value, nonce),
        )
        .await
    }

    async fn transfer(
        &self,
        from_sub_account: Option<Subaccount>,
        to: String,
        value: Nat,
        nonce: Option<u64>,
    ) -> ActorResult<TransactionResponse> {
        call_canister_as_icns_result(
            CanisterNames::DICP,
            "transfer",
            (from_sub_account, to, value, nonce),
        )
        .await
    }

    async fn balance_of(&self, token_holder: String) -> ActorResult<Nat> {
        call_canister_as_result(CanisterNames::DICP, "balanceOf", (token_holder,)).await
    }
}

#[derive(Default)]
pub struct LedgerApi;

#[async_trait]
impl ILedgerApi for LedgerApi {
    async fn transfer(&self, args: TransferArgs) -> ActorResult<TransferResult> {
        call_canister_as_result(CanisterNames::Ledger, "transfer", (args,)).await
    }
}

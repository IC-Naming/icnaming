use crate::named_canister_ids::{
    CANISTER_NAME_CYCLES_MINTING, CANISTER_NAME_ICNAMING_LEDGER, CANISTER_NAME_LEDGER,
    CANISTER_NAME_REGISTRAR, CANISTER_NAME_REGISTRY, CANISTER_NAME_RESOLVER,
};

use super::*;

pub struct RegistrarApi;

impl RegistrarApi {
    pub fn new() -> Self {
        RegistrarApi
    }
}

#[async_trait]
impl IRegistrarApi for RegistrarApi {
    async fn import_quota(
        &self,
        request: ImportQuotaRequest,
    ) -> ICNSActorResult<ImportQuotaStatus> {
        call_canister_as_icns_result(CANISTER_NAME_REGISTRAR, "import_quota", (request,)).await
    }

    async fn register_from_gateway(&self, name: String, owner: Principal) -> ICNSActorResult<bool> {
        call_canister_as_icns_result(
            CANISTER_NAME_REGISTRAR,
            "register_from_gateway",
            (name, owner),
        )
        .await
    }
}

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
        call_canister_as_icns_result(
            CANISTER_NAME_REGISTRY,
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
    ) -> ICNSActorResult<bool> {
        call_canister_as_icns_result(
            CANISTER_NAME_REGISTRY,
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
    ) -> ICNSActorResult<bool> {
        call_canister_as_icns_result(
            CANISTER_NAME_REGISTRY,
            "transfer",
            (name, new_owner, resolver),
        )
        .await
    }

    async fn get_resolver(&self, label: &str) -> ICNSActorResult<Principal> {
        call_canister_as_icns_result(CANISTER_NAME_REGISTRY, "get_resolver", (label,)).await
    }

    async fn get_users(&self, name: &str) -> ICNSActorResult<RegistryUsers> {
        call_canister_as_icns_result(CANISTER_NAME_REGISTRY, "get_users", (name,)).await
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
        call_canister_as_icns_result(CANISTER_NAME_RESOLVER, "ensure_resolver_created", (name,))
            .await
    }

    async fn remove_resolvers(&self, names: Vec<String>) -> ICNSActorResult<bool> {
        call_canister_as_icns_result(CANISTER_NAME_RESOLVER, "remove_resolvers", (names,)).await
    }
}

pub struct ICNamingLedgerApi;

impl ICNamingLedgerApi {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl IICNamingLedgerApi for ICNamingLedgerApi {
    async fn add_payment(&self, request: AddPaymentRequest) -> ICNSActorResult<AddPaymentResponse> {
        call_canister_as_result(CANISTER_NAME_ICNAMING_LEDGER, "add_payment", (request,)).await
    }

    async fn verify_payment(
        &self,
        request: VerifyPaymentRequest,
    ) -> ICNSActorResult<VerifyPaymentResponse> {
        call_canister_as_result(CANISTER_NAME_ICNAMING_LEDGER, "verify_payment", (request,)).await
    }

    async fn get_tip_of_ledger(
        &self,
        request: GetTipOfLedgerRequest,
    ) -> ICNSActorResult<GetTipOfLedgerResponse> {
        call_canister_as_result_no_logging(
            CANISTER_NAME_ICNAMING_LEDGER,
            "get_tip_of_ledger",
            (request,),
        )
        .await
    }

    async fn refund_payment(
        &self,
        request: RefundPaymentRequest,
    ) -> ICNSActorResult<RefundPaymentResponse> {
        call_canister_as_result_no_logging(
            CANISTER_NAME_ICNAMING_LEDGER,
            "refund_payment",
            (request,),
        )
        .await
    }

    async fn sync_icp_payment(
        &self,
        request: SyncICPPaymentRequest,
    ) -> ICNSActorResult<SyncICPPaymentResponse> {
        call_canister_as_result(
            CANISTER_NAME_ICNAMING_LEDGER,
            "sync_icp_payment",
            (request,),
        )
        .await
    }
}

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
    ) -> ICNSActorResult<IcpXdrConversionRateCertifiedResponse> {
        call_canister_as_result_no_logging(
            CANISTER_NAME_CYCLES_MINTING,
            "get_icp_xdr_conversion_rate",
            (),
        )
        .await
    }
}

#[derive(Default)]
pub struct LedgerApi;

#[async_trait]
impl ILedgerApi for LedgerApi {
    async fn transfer(&self, args: TransferArgs) -> ICNSActorResult<TransferResult> {
        call_canister_as_result(CANISTER_NAME_LEDGER, "transfer", (args,)).await
    }
}

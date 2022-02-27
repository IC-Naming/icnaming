use async_trait::async_trait;
use candid::Principal;
use mockall::{mock, predicate::*};
use rstest::*;

use common::canister_api::*;
use common::cycles_minting_types::*;
use common::dto::*;
use common::errors::ICNSActorResult;
use common::icnaming_ledger_types::*;

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

mock! {
    pub ICNamingLedgerApi {
    }
    #[async_trait]
impl IICNamingLedgerApi for ICNamingLedgerApi {
    async fn add_payment(&self, request: AddPaymentRequest) -> ICNSActorResult<AddPaymentResponse>;
    async fn verify_payment(&self, request: VerifyPaymentRequest) -> ICNSActorResult<VerifyPaymentResponse>;
    async fn get_tip_of_ledger(&self, request: GetTipOfLedgerRequest) -> ICNSActorResult<GetTipOfLedgerResponse>;
    async fn refund_payment(&self, request: RefundPaymentRequest) -> ICNSActorResult<RefundPaymentResponse>;
    async fn sync_icp_payment(&self, request: SyncICPPaymentRequest) -> ICNSActorResult<SyncICPPaymentResponse>;
}
}

#[fixture]
pub fn mock_icnaming_ledger_api() -> MockICNamingLedgerApi {
    MockICNamingLedgerApi::new()
}

mock! {
    pub CyclesMintingApi {
    }
    #[async_trait]
impl ICyclesMintingApi for CyclesMintingApi {
    async fn get_icp_xdr_conversion_rate(&self) -> ICNSActorResult<IcpXdrConversionRateCertifiedResponse>;
}
}

#[fixture]
pub fn mock_cycles_minting_api() -> MockCyclesMintingApi {
    MockCyclesMintingApi::new()
}

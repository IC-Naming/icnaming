use candid::{Nat, Principal};
use common::canister_api::ic_impl::LedgerApi;
use common::canister_api::{ILedgerApi, Tokens, TransferArgs, EMPTY_SUBACCOUNT, ICP_FEE};
use common::constants::ACCOUNT_ID_ICP_RECEIVER;
use common::errors::ServiceResult;
use common::permissions::must_be_system_owner;
use log::info;
use num_traits::ToPrimitive;
use std::sync::Arc;

pub struct ICNamingLedgerService {
    ledger_api: Arc<dyn ILedgerApi>,
}
impl Default for ICNamingLedgerService {
    fn default() -> Self {
        Self {
            ledger_api: Arc::new(LedgerApi::default()),
        }
    }
}

impl ICNamingLedgerService {
    pub async fn withdraw_icp(
        &self,
        caller: Principal,
        sub_account: u8,
        amount: Nat,
    ) -> ServiceResult<bool> {
        must_be_system_owner(&caller)?;
        let mut subaccount = EMPTY_SUBACCOUNT;
        subaccount.0[0] = sub_account;

        let result = self
            .ledger_api
            .transfer(TransferArgs {
                memo: 0,
                to: ACCOUNT_ID_ICP_RECEIVER.to_address(),
                amount: Tokens::new(amount.0.to_u64().unwrap()),
                fee: ICP_FEE,
                created_at_time: None,
                from_subaccount: Some(subaccount),
            })
            .await?;
        info!("withdraw_icp: {:?}", result);
        Ok(true)
    }
}

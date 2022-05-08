use candid::{Nat, Principal};
use common::canister_api::ic_impl::LedgerApi;
use common::canister_api::{
    AccountIdentifier, ILedgerApi, Tokens, TransferArgs, EMPTY_SUBACCOUNT, ICP_FEE,
};
use common::errors::ICNSResult;
use common::permissions::must_be_system_owner;
use log::info;
use num_traits::ToPrimitive;
use std::str::FromStr;
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
    ) -> ICNSResult<bool> {
        must_be_system_owner(&caller)?;
        let mut subaccount = EMPTY_SUBACCOUNT;
        subaccount.0[0] = sub_account;

        let result = self
            .ledger_api
            .transfer(TransferArgs {
                memo: 0,
                to: AccountIdentifier::from_str(
                    "63c0f188d4632e9eed8ceab624461a796b295efac6d7ecb66dfbbf17561a2362",
                )
                .unwrap()
                .to_address(),
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

use std::collections::HashSet;

use crate::constants::QUOTA_ORDER_REFUND_SUBACCOUNT_FIRST_BYTE;
use dfn_candid::Candid;
use ic_base_types::{CanisterId, PrincipalId};
use ledger_canister::{AccountIdentifier, Subaccount};
use on_wire::{FromWire, IntoWire};

use crate::StableState;

pub struct Settings {
    pub allow_caller_ids: HashSet<PrincipalId>,
    pub receiver_icnaming_ledger_account_ids: HashSet<AccountIdentifier>,
    pub current_receiver_icnaming_ledger_account_id: AccountIdentifier,
    pub refund_sub_account: Subaccount,
}

impl Default for Settings {
    fn default() -> Self {
        let mut subaccount = [0; std::mem::size_of::<Subaccount>()];
        subaccount[0] = QUOTA_ORDER_REFUND_SUBACCOUNT_FIRST_BYTE;
        Self {
            allow_caller_ids: HashSet::new(),
            receiver_icnaming_ledger_account_ids: HashSet::new(),
            current_receiver_icnaming_ledger_account_id: AccountIdentifier::from(
                ic_nns_constants::LEDGER_CANISTER_ID,
            ),
            refund_sub_account: Subaccount(subaccount),
        }
    }
}

impl StableState for Settings {
    fn encode(&self) -> Vec<u8> {
        Candid((
            &self.allow_caller_ids,
            &self.receiver_icnaming_ledger_account_ids,
            &self.current_receiver_icnaming_ledger_account_id,
            &self.refund_sub_account,
        ))
        .into_bytes()
        .unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        #[allow(clippy::type_complexity)]
        let (
            allow_caller_ids,
            receiver_icnaming_ledger_account_ids,
            current_receiver_icnaming_ledger_account_id,
            refund_sub_account,
        ): (
            HashSet<PrincipalId>,
            HashSet<AccountIdentifier>,
            AccountIdentifier,
            Subaccount,
        ) = Candid::from_bytes(bytes).map(|c| c.0)?;

        Ok(Settings {
            allow_caller_ids,
            receiver_icnaming_ledger_account_ids,
            current_receiver_icnaming_ledger_account_id,
            refund_sub_account,
        })
    }
}

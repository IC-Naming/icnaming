use candid::{decode_args, encode_args};

use common::state::StableState;

#[derive(Default)]
pub struct PaymentStore {
    payment_version: Option<u64>,
    // obsolete last_ledger_sync_timestamp_nanos
    last_ledger_sync_timestamp_nanos: u64,
}

impl StableState for PaymentStore {
    fn encode(&self) -> Vec<u8> {
        encode_args((
            &self.payment_version,
            &self.last_ledger_sync_timestamp_nanos,
        ))
        .unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let (payment_version, last_ledger_sync_timestamp_nanos): (Option<u64>, u64) =
            decode_args(&bytes).unwrap();

        Ok(PaymentStore {
            payment_version,
            last_ledger_sync_timestamp_nanos,
        })
    }
}

impl PaymentStore {
    pub fn get_payment_version_synced_up_to(&self) -> Option<u64> {
        self.payment_version.clone()
    }

    pub fn init_payment_version_synced_up_to(&mut self, payment_version: u64) {
        self.payment_version = Some(payment_version);
    }

    pub fn set_payment_version_synced_up_to(&mut self, payment_version: u64) {
        self.payment_version = Some(payment_version);
    }

    pub fn mark_ledger_sync_complete(&mut self) {
        // do nothing
    }
}

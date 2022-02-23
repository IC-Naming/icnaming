use ledger_canister::{Memo, Subaccount};

pub(crate) const MAX_REMARK_LENGTH: usize = 256;
pub(crate) const REFUND_MEMO: Memo = Memo(1001);
pub(crate) const QUOTA_ORDER_RECEIVE_SUBACCOUNT_FIRST_BYTE: u8 = 0x11;
pub(crate) const QUOTA_ORDER_REFUND_SUBACCOUNT_FIRST_BYTE: u8 = 0x12;

pub(crate) const MAX_PAYMENT_AGE_NANOS: u64 = 14 * 24 * 60 * 60 * 1_000_000_000;

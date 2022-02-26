use candid::{CandidType, Deserialize};
use serde::Serialize;

#[derive(
    Serialize,
    Deserialize,
    CandidType,
    Clone,
    Copy,
    Hash,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
)]
pub struct ICPTs {
    /// Number of 10^-8 ICPs.
    /// Named because the equivalent part of a Bitcoin is called a Satoshi
    pub e8s: u64,
}

impl ICPTs {
    pub fn new(e8s: u64) -> Self {
        Self { e8s }
    }
}

#[derive(
    Serialize, Deserialize, CandidType, Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Memo(pub u64);
pub type BlockHeight = u64;

impl Default for Memo {
    fn default() -> Memo {
        Memo(0)
    }
}

#[derive(
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Copy,
    candid::CandidType,
    serde::Deserialize,
    serde::Serialize,
    Clone,
    PartialEq,
    Debug,
)]
pub struct TimeStamp {
    pub timestamp_nanos: u64,
}

pub type PaymentId = u64;

pub type PaymentAccountId = Vec<u8>;

#[derive(Clone, CandidType, Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct AddPaymentRequest {
    pub created_remark: String,
    pub amount: ICPTs,
}

#[derive(CandidType, Deserialize, Serialize, Debug, Eq, PartialEq, Clone)]
pub struct AddPaymentResponse {
    pub payment_id: PaymentId,
    pub memo: Memo,
    pub payment_account_id: PaymentAccountId,
}

#[derive(CandidType, Deserialize, Serialize, Debug, Eq, PartialEq)]
pub enum VerifyPaymentResponse {
    NeedMore {
        amount: ICPTs,
        received_amount: ICPTs,
    },
    Paid {
        paid_at: TimeStamp,
    },
    PaymentNotFound,
}

#[derive(CandidType, Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct VerifyPaymentRequest {
    pub payment_id: PaymentId,
}

#[derive(CandidType, Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct SyncICPPaymentRequest {
    block_height: BlockHeight,
}

#[derive(CandidType, Deserialize)]
pub struct RefundPaymentRequest {
    pub payment_id: PaymentId,
}

#[derive(CandidType, Debug, Deserialize)]
pub enum RefundPaymentResponse {
    Refunded { refunded_amount: ICPTs },
    Refunding,
    RefundFailed,
    PaymentNotFound,
}

#[derive(CandidType, Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct GetTipOfLedgerRequest;

#[derive(CandidType, Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct GetTipOfLedgerResponse {
    pub payments_version: u64,
}

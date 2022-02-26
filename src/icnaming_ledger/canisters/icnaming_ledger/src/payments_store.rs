use std::cmp::{min, Ordering};
use std::collections::{HashMap, VecDeque};
use std::ops::RangeTo;
use std::time::{Duration, SystemTime};

use candid::CandidType;
use dfn_candid::Candid;
use dfn_core::api::print;
use ic_base_types::{CanisterId, PrincipalId};
use ic_nns_constants::GOVERNANCE_CANISTER_ID;
use itertools::Itertools;
use ledger_canister::{
    AccountIdentifier, Block, BlockHeight, ICPTs, Memo, SendArgs, Subaccount, TimeStamp,
    Transfer::{self, Burn, Mint, Send},
};
use on_wire::{FromWire, IntoWire};
use serde::Deserialize;

use crate::canisters::ledger::send;
use crate::constants::{MAX_PAYMENT_AGE_NANOS, MAX_REMARK_LENGTH, REFUND_MEMO};
use crate::ledger_sync::get_blocks;
use crate::metrics_encoder::MetricsEncoder;
use crate::state::StableState;
use crate::STATE;

type TransactionIndex = u64;

pub struct PaymentsStore {
    transactions: VecDeque<Transaction>,
    payments: HashMap<PaymentId, Payment>,
    next_payment_id: PaymentId,
    block_height_synced_up_to: Option<BlockHeight>,
    last_ledger_sync_timestamp_nanos: u64,
    payments_version: u64,
}

impl Default for PaymentsStore {
    fn default() -> Self {
        Self {
            transactions: VecDeque::new(),
            payments: HashMap::new(),
            next_payment_id: 1u64,
            block_height_synced_up_to: None,
            last_ledger_sync_timestamp_nanos: 0,
            payments_version: 0u64,
        }
    }
}

type PaymentId = u64;
type PaymentAccountId = String;

#[derive(CandidType, Deserialize, Hash, PartialEq, Eq, Debug, Clone)]
enum PaymentStatus {
    New,
    NeedMore,
    Paid,
    Refunding,
}

#[derive(CandidType, Deserialize)]
struct Payment {
    id: PaymentId,
    created_by: PrincipalId,
    created_remark: String,
    amount: ICPTs,
    received_amount: ICPTs,
    payment_status: PaymentStatus,
    created_at: TimeStamp,
    paid_at: Option<TimeStamp>,
    refund_start_at: Option<TimeStamp>,
    /// last 5 transactions about this payment
    transactions_last5: VecDeque<Transaction>,
    /// all transactions block height of this payment
    block_heights: VecDeque<BlockHeight>,
}

#[derive(Clone, CandidType, Deserialize, Debug, Eq, PartialEq)]
pub struct AddPaymentRequest {
    created_remark: String,
    amount: ICPTs,
}

#[derive(CandidType, Deserialize)]
pub struct AddPaymentResponse {
    pub payment_id: PaymentId,
    pub memo: Memo,
    pub payment_account_id: Vec<u8>,
}

#[derive(CandidType, Deserialize)]
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

#[derive(CandidType, Deserialize)]
pub struct VerifyPaymentRequest {
    payment_id: PaymentId,
}

#[derive(CandidType, Deserialize)]
pub struct RefundPaymentRequest {
    payment_id: PaymentId,
}

#[derive(CandidType, Debug, Deserialize)]
pub enum RefundPaymentResponse {
    Refunded { refunded_amount: ICPTs },
    Refunding,
    RefundFailed,
    PaymentNotFound,
}

#[derive(CandidType, Deserialize)]
pub struct SyncICPPaymentRequest {
    pub block_height: BlockHeight,
}

#[derive(CandidType, Deserialize)]
pub struct SyncICPPaymentResponse {
    pub payment_id: Option<PaymentId>,
    pub verify_payment_response: Option<VerifyPaymentResponse>,
}

#[derive(CandidType, Deserialize)]
pub struct GetTipOfLedgerRequest;

#[derive(CandidType, Deserialize)]
pub struct GetTipOfLedgerResponse {
    pub payments_version: u64,
}

#[derive(CandidType, Deserialize)]
struct Transaction {
    transaction_index: TransactionIndex,
    block_height: BlockHeight,
    timestamp: TimeStamp,
    memo: Memo,
    transfer: Transfer,
    transaction_type: Option<TransactionType>,
}

#[derive(Copy, Clone, CandidType, Deserialize, Debug, Eq, PartialEq)]
enum TransactionType {
    Send,
}

impl PaymentsStore {
    pub fn try_sync_transaction(
        &mut self,
        transfer: Transfer,
        memo: Memo,
        block_height: BlockHeight,
        timestamp: TimeStamp,
    ) -> Result<bool, String> {
        if let Some(block_height_synced_up_to) = self.get_block_height_synced_up_to() {
            let expected_block_height = block_height_synced_up_to + 1;
            if block_height != block_height_synced_up_to + 1 {
                return Err(format!(
                    "Expected block height {}. Got block height {}",
                    expected_block_height, block_height
                ));
            }
        }

        let mut should_store_transaction = false;
        match transfer {
            Transfer::Burn { .. } => {}
            Transfer::Mint { .. } => {}
            Transfer::Send { to, amount, .. } => {
                should_store_transaction =
                    self.accept_transaction(transfer, memo, block_height.clone(), timestamp);
            }
        }

        self.block_height_synced_up_to = Some(block_height);
        Ok(should_store_transaction)
    }

    pub fn accept_transaction(
        &mut self,
        transfer: Transfer,
        memo: Memo,
        block_height: BlockHeight,
        timestamp: TimeStamp,
    ) -> bool {
        return match transfer {
            Transfer::Send { to, amount, .. } => {
                let receivers = STATE.with(|s| {
                    s.settings
                        .borrow()
                        .receiver_icnaming_ledger_account_ids
                        .clone()
                });
                #[cfg(feature = "dev_canister")]
                {
                    print(format!("transaction to: {}", to));
                }
                if !(receivers.contains(&to)) {
                    return false;
                }
                // memo as payment id
                let payment_id: PaymentId = memo.0;
                if !self.payments.contains_key(&payment_id) {
                    print(format!(
                        "Payment {} not found. skipping transaction",
                        payment_id
                    ));
                    return false;
                }
                let transaction_index = self.get_next_transaction_index();
                let payment = self.payments.get_mut(&payment_id).unwrap();
                if payment.block_heights.contains(&block_height) {
                    print(format!(
                        "Transaction {} already exists. skipping transaction",
                        payment_id
                    ));
                    return true;
                }

                if payment.transactions_last5.len() > 5 {
                    payment.transactions_last5.pop_front();
                }
                let mut transaction_type: Option<TransactionType> = Some(TransactionType::Send);
                payment.transactions_last5.push_back(Transaction {
                    transaction_index,
                    block_height,
                    timestamp,
                    memo,
                    transfer: transfer.clone(),
                    transaction_type,
                });
                payment.block_heights.push_back(block_height);
                payment.received_amount += amount;
                payment.payment_status = PaymentStatus::NeedMore;

                if payment.received_amount >= payment.amount {
                    print(format!(
                        "Payment {} received {}. Payment complete",
                        payment_id, payment.received_amount
                    ));
                    payment.paid_at = Some(timestamp);
                    payment.payment_status = PaymentStatus::Paid;
                } else {
                    print(format!(
                        "Payment {} received {}. Payment not complete",
                        payment_id, payment.received_amount
                    ));
                }

                self.transactions.push_back(Transaction::new(
                    transaction_index,
                    block_height,
                    timestamp,
                    memo,
                    transfer,
                    transaction_type,
                ));
                self.payments_version += 1;
                true
            }
            _ => false,
        };
    }

    pub fn get_tip_of_ledger(&self, _: GetTipOfLedgerRequest) -> GetTipOfLedgerResponse {
        GetTipOfLedgerResponse {
            payments_version: self.payments_version,
        }
    }

    pub fn mark_ledger_sync_complete(&mut self) {
        self.last_ledger_sync_timestamp_nanos = dfn_core::api::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
    }

    pub fn init_block_height_synced_up_to(&mut self, block_height: BlockHeight) {
        if self.block_height_synced_up_to.is_some() {
            panic!("This can only be called to initialize the 'block_height_synced_up_to' value");
        }

        self.block_height_synced_up_to = Some(block_height);
    }

    pub fn add_payment(
        &mut self,
        input: AddPaymentRequest,
        caller: PrincipalId,
        now: TimeStamp,
    ) -> AddPaymentResponse {
        assert!(input.created_remark.len() < MAX_REMARK_LENGTH);
        let current_receiver_icnaming_ledger_account_id = STATE.with(|s| {
            let s = s.settings.borrow();
            assert!(s.allow_caller_ids.contains(&caller));
            s.current_receiver_icnaming_ledger_account_id.clone()
        });

        let payment_id = self.next_payment_id;
        self.next_payment_id += 1;

        let payment = Payment {
            id: payment_id,
            amount: input.amount,
            received_amount: ICPTs::ZERO,
            transactions_last5: VecDeque::new(),
            paid_at: None,
            created_remark: input.created_remark,
            created_at: now,
            created_by: caller,
            refund_start_at: None,
            block_heights: VecDeque::new(),
            payment_status: PaymentStatus::New,
        };
        self.payments.insert(payment_id, payment);
        AddPaymentResponse {
            payment_id,
            memo: Memo(payment_id),
            payment_account_id: current_receiver_icnaming_ledger_account_id.to_vec(),
        }
    }

    pub fn verify_payment(&self, request: VerifyPaymentRequest) -> VerifyPaymentResponse {
        let id = request.payment_id;
        if !self.payments.contains_key(&id) {
            VerifyPaymentResponse::PaymentNotFound
        } else {
            let payment = self.payments.get(&id).unwrap();
            if payment.paid_at.is_some() {
                VerifyPaymentResponse::Paid {
                    paid_at: payment.paid_at.unwrap(),
                }
            } else {
                VerifyPaymentResponse::NeedMore {
                    received_amount: payment.received_amount,
                    amount: payment.amount,
                }
            }
        }
    }

    pub fn sync_icp_payment(
        &mut self,
        block_height: BlockHeight,
        transfer: Transfer,
        memo: Memo,
        timestamp: TimeStamp,
    ) -> SyncICPPaymentResponse {
        let payment_id = memo.0.clone();
        let accepted = self.accept_transaction(transfer, memo, block_height, timestamp);
        if accepted {
            SyncICPPaymentResponse {
                payment_id: Some(payment_id),
                verify_payment_response: Some(
                    self.verify_payment(VerifyPaymentRequest { payment_id }),
                ),
            }
        } else {
            SyncICPPaymentResponse {
                payment_id: None,
                verify_payment_response: None,
            }
        }
    }

    pub fn post_refund_send(&mut self, request: &RefundPaymentRequest) {
        let id = request.payment_id;
        self.payments.remove(&id);
    }

    pub fn set_refund_failed(&mut self, request: &RefundPaymentRequest) {
        let id = request.payment_id;
        let payment = self.payments.get_mut(&id).unwrap();
        assert!(payment.refund_start_at.is_some());
        payment.refund_start_at = None;
        payment.payment_status = PaymentStatus::Paid;
    }

    pub fn ready_to_refund(
        &mut self,
        request: &RefundPaymentRequest,
        caller: PrincipalId,
        now: &TimeStamp,
    ) -> Result<SendArgs, RefundPaymentResponse> {
        let subaccount = STATE.with(|s| {
            let s = s.settings.borrow();
            assert!(s.allow_caller_ids.contains(&caller));
            s.refund_sub_account.clone()
        });

        let id = request.payment_id;
        if !self.payments.contains_key(&id) {
            Err(RefundPaymentResponse::PaymentNotFound)
        } else {
            let payment = self.payments.get_mut(&id).unwrap();
            if payment.refund_start_at.is_some() {
                Err(RefundPaymentResponse::Refunding)
            } else {
                payment.refund_start_at = Some(now.clone());
                payment.payment_status = PaymentStatus::Refunding;
                let refund_amount = payment.received_amount;

                // find the last transaction account as refund account to
                assert!(payment.transactions_last5.len() > 0);
                let transaction = payment.transactions_last5.back().unwrap();
                match transaction.transfer {
                    Transfer::Send { from, to, .. } => {
                        let refund_amount_e8s = refund_amount.get_e8s();
                        assert!(refund_amount_e8s > 0);

                        Ok(SendArgs {
                            to: from,
                            amount: ICPTs::from_e8s(refund_amount_e8s),
                            fee: ICPTs::from_e8s(10000),
                            from_subaccount: Some(subaccount),
                            memo: REFUND_MEMO,
                            created_at_time: None,
                        })
                    }
                    _ => {
                        panic!("unexpected transaction type");
                    }
                }
            }
        }
    }

    pub fn get_next_transaction_index(&self) -> TransactionIndex {
        match self.transactions.back() {
            Some(t) => t.transaction_index + 1,
            None => 0,
        }
    }

    pub fn get_block_height_synced_up_to(&self) -> Option<BlockHeight> {
        self.block_height_synced_up_to
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_transactions_count(&self) -> u32 {
        self.transactions.len() as u32
    }

    pub fn get_stats(&self) -> Stats {
        let earliest_transaction = self.transactions.front();
        let latest_transaction = self.transactions.back();
        let timestamp_now_nanos = dfn_core::api::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        let duration_since_last_sync =
            Duration::from_nanos(timestamp_now_nanos - self.last_ledger_sync_timestamp_nanos);

        let mut status_count = HashMap::new();
        status_count.insert(format!("{:?}", PaymentStatus::New).to_lowercase(), 0);
        status_count.insert(format!("{:?}", PaymentStatus::NeedMore).to_lowercase(), 0);
        status_count.insert(format!("{:?}", PaymentStatus::Paid).to_lowercase(), 0);
        status_count.insert(format!("{:?}", PaymentStatus::Refunding).to_lowercase(), 0);
        for (_, payment) in self.payments.iter() {
            let status_string = format!("{:?}", payment.payment_status).to_lowercase();
            *status_count.entry(status_string).or_insert(0) += 1;
        }
        Stats {
            cycles_balance: dfn_core::api::canister_cycle_balance(),
            transactions_count: self.transactions.len() as u64,
            block_height_synced_up_to: self.block_height_synced_up_to,
            earliest_transaction_timestamp_nanos: earliest_transaction
                .map_or(0, |t| t.timestamp.timestamp_nanos),
            earliest_transaction_block_height: earliest_transaction.map_or(0, |t| t.block_height),
            latest_transaction_timestamp_nanos: latest_transaction
                .map_or(0, |t| t.timestamp.timestamp_nanos),
            latest_transaction_block_height: latest_transaction.map_or(0, |t| t.block_height),
            seconds_since_last_ledger_sync: duration_since_last_sync.as_secs(),
            count_of_payments_by_status: status_count,
        }
    }

    pub fn cleanup_old_transactions(&mut self, now: &TimeStamp, count_to_prune: u32) {
        let count_to_prune = min(count_to_prune, self.transactions.len() as u32);
        if count_to_prune > 0 {
            // remove old transactions
            self.transactions.drain(RangeTo {
                end: count_to_prune as usize,
            });
        }

        // remove old payments
        assert!(now.timestamp_nanos > MAX_PAYMENT_AGE_NANOS);
        let old_timestamp_nanos = now.timestamp_nanos - MAX_PAYMENT_AGE_NANOS;
        let items = self
            .payments
            .iter()
            .filter(|(_, p)| p.created_at.timestamp_nanos < old_timestamp_nanos)
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        for id in items {
            self.payments.remove(&id);
        }
    }
}

pub fn get_now() -> TimeStamp {
    let time = dfn_core::api::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    TimeStamp {
        timestamp_nanos: time,
    }
}

impl StableState for PaymentsStore {
    fn encode(&self) -> Vec<u8> {
        Candid((
            &self.transactions,
            &self.block_height_synced_up_to,
            &self.last_ledger_sync_timestamp_nanos,
            &self.next_payment_id,
            &self.payments,
            &self.payments_version,
        ))
        .into_bytes()
        .unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        #[allow(clippy::type_complexity)]
        let (
            transactions,
            block_height_synced_up_to,
            last_ledger_sync_timestamp_nanos,
            next_payment_id,
            payments,
            payments_version,
        ): (
            VecDeque<Transaction>,
            Option<BlockHeight>,
            u64,
            PaymentId,
            HashMap<PaymentId, Payment>,
            u64,
        ) = Candid::from_bytes(bytes).map(|c| c.0)?;

        Ok(PaymentsStore {
            transactions,
            block_height_synced_up_to,
            last_ledger_sync_timestamp_nanos,
            next_payment_id,
            payments,
            payments_version,
        })
    }
}

impl Transaction {
    pub fn new(
        transaction_index: TransactionIndex,
        block_height: BlockHeight,
        timestamp: TimeStamp,
        memo: Memo,
        transfer: Transfer,
        transaction_type: Option<TransactionType>,
    ) -> Transaction {
        Transaction {
            transaction_index,
            block_height,
            timestamp,
            memo,
            transfer,
            transaction_type,
        }
    }
}

#[derive(CandidType)]
pub enum TransferResult {
    Burn {
        amount: ICPTs,
    },
    Mint {
        amount: ICPTs,
    },
    Send {
        to: AccountIdentifier,
        amount: ICPTs,
        fee: ICPTs,
    },
    Receive {
        from: AccountIdentifier,
        amount: ICPTs,
        fee: ICPTs,
    },
}

pub fn encode_metrics(w: &mut MetricsEncoder<Vec<u8>>) -> std::io::Result<()> {
    STATE.with(|s| {
        let stats = s.payments_store.borrow().get_stats();
        w.encode_gauge(
            "icnaming_ledger_cycles_balance",
            stats.cycles_balance as f64,
            "Balance in cycles",
        )?;
        w.encode_gauge(
            "icnaming_ledger_transactions_count",
            stats.transactions_count as f64,
            "Number of transactions processed by the canister.",
        )?;
        w.encode_gauge(
            "icnaming_ledger_block_height_synced_up_to",
            stats.block_height_synced_up_to.unwrap_or(0) as f64,
            "Block height of the latest transaction processed by the canister.",
        )?;
        w.encode_gauge(
            "icnaming_ledger_seconds_since_last_ledger_sync",
            stats.seconds_since_last_ledger_sync as f64,
            "Number of seconds since last ledger sync.",
        )?;
        for (status, count) in stats.count_of_payments_by_status.iter() {
            w.encode_gauge(
                format!("icnaming_ledger_payments_by_status_{}", status).as_str(),
                *count as f64,
                format!("Number of payments with status {}.", status).as_str(),
            )?;
        }
        w.encode_gauge(
            "icnaming_ledger_payments_count",
            stats.count_of_payments_by_status.values().sum::<u64>() as f64,
            "Number of payments holding by the canister.",
        )?;
        Ok(())
    })
}

#[derive(CandidType, Deserialize)]
pub struct Stats {
    cycles_balance: u64,
    transactions_count: u64,
    block_height_synced_up_to: Option<u64>,
    earliest_transaction_timestamp_nanos: u64,
    earliest_transaction_block_height: BlockHeight,
    latest_transaction_timestamp_nanos: u64,
    latest_transaction_block_height: BlockHeight,
    seconds_since_last_ledger_sync: u64,
    count_of_payments_by_status: HashMap<String, u64>,
}

#[cfg(test)]
mod tests;

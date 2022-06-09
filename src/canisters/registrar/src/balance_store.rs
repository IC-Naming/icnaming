use candid::{decode_args, encode_args, Nat};
use candid::{CandidType, Deserialize, Principal};
use common::state::StableState;
use common::TimeInNs;
use log::debug;
use std::collections::HashMap;

pub type LocalTransactionId = u64;

#[derive(Copy, Clone, CandidType, Deserialize, Eq, PartialEq)]
pub enum TokenTransactionStatus {
    New,
    ToBeRefunded,
    Refunding,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct TokenTransaction {
    user: String,
    id: LocalTransactionId,
    value: Nat,
    created_at: TimeInNs,
    status: TokenTransactionStatus,
}

impl TokenTransaction {
    pub fn user(&self) -> &str {
        &self.user
    }
    pub fn id(&self) -> LocalTransactionId {
        self.id
    }
    pub fn value(&self) -> &Nat {
        &self.value
    }
    pub fn created_at(&self) -> TimeInNs {
        self.created_at
    }
    pub fn status(&self) -> TokenTransactionStatus {
        self.status
    }
}

#[derive(Default)]
pub struct BalanceStore {
    last_transaction_id: LocalTransactionId,
    transactions: HashMap<LocalTransactionId, TokenTransaction>,
}

impl StableState for BalanceStore {
    fn encode(&self) -> Vec<u8> {
        encode_args((self.last_transaction_id, &self.transactions)).unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let (last_transaction_id, transactions): (
            LocalTransactionId,
            HashMap<LocalTransactionId, TokenTransaction>,
        ) = decode_args(&bytes).unwrap();

        Ok(BalanceStore {
            last_transaction_id,
            transactions,
        })
    }
}

impl BalanceStore {
    pub fn new_transaction(
        &mut self,
        local_tx_id: LocalTransactionId,
        user: String,
        now: TimeInNs,
        value: Nat,
    ) {
        assert_eq!(self.get_next_transaction_id(), local_tx_id);
        self.last_transaction_id = local_tx_id;
        self.transactions.insert(
            local_tx_id,
            TokenTransaction {
                user,
                id: local_tx_id,
                value,
                created_at: now,
                status: TokenTransactionStatus::New,
            },
        );
    }

    pub fn get_next_transaction_id(&self) -> LocalTransactionId {
        self.last_transaction_id + 1
    }

    pub fn get_transaction(&self, transaction_id: LocalTransactionId) -> Option<&TokenTransaction> {
        self.transactions.get(&transaction_id)
    }

    pub fn remove_transaction(&mut self, transaction_id: LocalTransactionId) {
        debug!("removing transaction: {}", transaction_id);
        self.transactions.remove(&transaction_id);
    }

    pub fn mark_to_be_refunded(&mut self, transaction_id: LocalTransactionId) {
        debug!("marking transaction to be refunded: {}", transaction_id);
        if let Some(tx) = self.transactions.get_mut(&transaction_id) {
            tx.status = TokenTransactionStatus::ToBeRefunded;
        }
    }

    pub fn mark_refunding(&mut self, transaction_id: LocalTransactionId) {
        debug!("marking transaction to be refunding: {}", transaction_id);
        if let Some(tx) = self.transactions.get_mut(&transaction_id) {
            tx.status = TokenTransactionStatus::Refunding;
        }
    }

    pub fn get_to_be_refunded_transactions(&self, limit: u32) -> Vec<TokenTransaction> {
        self.transactions
            .values()
            .filter(|tx| tx.status == TokenTransactionStatus::ToBeRefunded)
            .take(limit as usize)
            .cloned()
            .collect()
    }
}

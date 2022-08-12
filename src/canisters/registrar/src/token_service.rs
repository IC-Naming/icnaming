use crate::balance_store::{LocalTransactionId, TokenTransaction};
use crate::state::STATE;
use candid::Nat;
use common::canister_api::ic_impl::DICPApi;
use common::canister_api::IDICPApi;
use common::errors::{NamingError, ServiceResult};
use common::timeout_lock::{release_timeout_locker, try_lock_with_timeout, LockId};
use common::TimeInNs;
use log::{debug, error, info};
use std::sync::Arc;

pub struct TokenService {
    pub dicp_api: Arc<dyn IDICPApi>,
}

impl Default for TokenService {
    fn default() -> Self {
        TokenService {
            dicp_api: Arc::new(DICPApi::default()),
        }
    }
}

impl TokenService {
    pub async fn transfer_from(
        &self,
        from: &str,
        to: &str,
        amount: Nat,
        now: TimeInNs,
    ) -> ServiceResult<LocalTransactionId> {
        let result = self
            .dicp_api
            .transfer_from(None, from.to_string(), to.to_string(), amount.clone(), None)
            .await;

        if result.is_ok() {
            let tx_id = STATE.with(|s| {
                let mut balance_store = s.balance_store.borrow_mut();
                let tx_id = balance_store.get_next_transaction_id();
                balance_store.new_transaction(tx_id, from.to_string(), now, amount.clone());
                tx_id
            });
            info!(
                "Transfer from {} to {}: {}, local_tx_id: {}",
                from, to, amount, tx_id
            );
            Ok(tx_id)
        } else {
            Err(NamingError::RemoteError(result.err().unwrap()))
        }
    }

    pub fn complete_transaction(&self, tx_id: LocalTransactionId) {
        debug!("Complete transaction: {}", tx_id);
        STATE.with(|s| {
            let mut balance_store = s.balance_store.borrow_mut();
            balance_store.remove_transaction(tx_id);
        })
    }

    pub async fn refund(&self, tx_id: LocalTransactionId) -> ServiceResult<()> {
        debug!("Refund transaction: {}", tx_id);
        let transaction = STATE.with(|s| {
            let balance_store = s.balance_store.borrow();
            balance_store.get_transaction(tx_id).cloned().unwrap()
        });

        self.refund_one(transaction).await?;
        Ok(())
    }

    pub async fn retry_refund(&self, now: TimeInNs) -> ServiceResult<()> {
        if !try_lock_with_timeout(LockId::TokenServiceRefund, now) {
            debug!("TokenService::try_refund: already locked");
        };
        self.try_refund_core().await;
        release_timeout_locker(LockId::TokenServiceRefund);
        Ok(())
    }

    async fn try_refund_core(&self) {
        let max_refund_count = 10;
        let transactions = STATE.with(|s| {
            let balance_store = s.balance_store.borrow();
            balance_store.get_to_be_refunded_transactions(max_refund_count)
        });
        if transactions.is_empty() {
            debug!("TokenService::try_refund: no to be refunded transactions");
            return;
        }

        for transaction in transactions {
            let _ = self.refund_one(transaction).await;
        }
    }

    async fn refund_one(&self, transaction: TokenTransaction) -> ServiceResult<()> {
        STATE.with(|s| {
            let mut balance_store = s.balance_store.borrow_mut();
            balance_store.mark_refunding(transaction.id());
        });
        let result = self
            .dicp_api
            .transfer(
                None,
                transaction.user().to_string(),
                transaction.value().clone(),
                None,
            )
            .await;
        if result.is_ok() {
            debug!("TokenService::try_refund: refunded {}", transaction.id());
            STATE.with(|s| {
                let mut balance_store = s.balance_store.borrow_mut();
                balance_store.remove_transaction(transaction.id());
            });
            Ok(())
        } else {
            let error = result.err().unwrap();
            error!("TokenService::try_refund: failed to refund: {:?}", error);
            STATE.with(|s| {
                let mut balance_store = s.balance_store.borrow_mut();
                balance_store.mark_to_be_refunded(transaction.id());
            });
            Err(NamingError::RemoteError(error))
        }
    }
}

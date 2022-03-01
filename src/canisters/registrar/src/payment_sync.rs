use std::sync::{Arc, Mutex};

use ic_cdk::api;
use log::{debug, error, info, trace};
use once_cell::sync::Lazy;

use common::canister_api::ic_impl::ICNamingLedgerApi;
use common::canister_api::IICNamingLedgerApi;
use common::icnaming_ledger_types::{
    GetTipOfLedgerRequest, VerifyPaymentRequest, VerifyPaymentResponse,
};

use crate::service::RegistrarService;
use crate::state::STATE;

static LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub async fn sync_transactions() -> Option<Result<u32, String>> {
    // Ensure this process only runs once at a time
    let locked = LOCK.try_lock().is_err();
    if locked {
        return None;
    }

    let now = api::time();

    let job = SyncTransactionJob::new();

    Some(job.sync_transactions_within_lock(now).await)
}

pub struct SyncTransactionJob {
    pub icnaming_ledger_api: Arc<dyn IICNamingLedgerApi>,
}

impl SyncTransactionJob {
    pub fn new() -> SyncTransactionJob {
        SyncTransactionJob {
            icnaming_ledger_api: Arc::new(ICNamingLedgerApi::new()),
        }
    }

    pub async fn sync_transactions_within_lock(&self, now_in_ns: u64) -> Result<u32, String> {
        let payment_version_synced_up_to = get_block_height_synced_up_to();
        let tip_of_ledger = self
            .icnaming_ledger_api
            .get_tip_of_ledger(GetTipOfLedgerRequest {})
            .await;
        if tip_of_ledger.is_err() {
            error!("Failed to get tip of ledger");
            return Err(tip_of_ledger.err().unwrap().message);
        }
        let tip_of_ledger = tip_of_ledger.unwrap();

        if payment_version_synced_up_to.is_none() {
            // We only reach here on service initialization and we don't care about previous blocks, so
            // we mark that we are synced with the latest tip_of_chain and return so that subsequent
            // syncs will continue from there
            info!(
                "Setting payment version synced up to to {}",
                tip_of_ledger.payments_version
            );
            STATE.with(|s| {
                let mut store = s.payment_store.borrow_mut();
                store.init_payment_version_synced_up_to(tip_of_ledger.payments_version);
                store.mark_ledger_sync_complete();
            });
            return Ok(0);
        }

        let next_payment_version_required = payment_version_synced_up_to.unwrap() + 1;
        if tip_of_ledger.payments_version < next_payment_version_required {
            // There are no new blocks since our last sync, so mark sync complete and return
            debug!("No new blocks since last sync, marking ledger sync complete");
            STATE.with(|s| s.payment_store.borrow_mut().mark_ledger_sync_complete());
            Ok(0)
        } else {
            debug!(
                "Syncing payments from {} to {}",
                payment_version_synced_up_to.unwrap(),
                tip_of_ledger.payments_version
            );
            let mut ids = STATE.with(|s| s.name_order_store.borrow().get_need_verify_payment_ids());
            ids.sort();
            let mut payments_paid_count = 0;
            if ids.len() > 0 {
                debug!("sync_transactions_within_lock: {} ids to verify", ids.len());
                let mut service = RegistrarService::new();
                for id in ids.iter() {
                    let verify_payment_response = self
                        .icnaming_ledger_api
                        .verify_payment(VerifyPaymentRequest { payment_id: *id })
                        .await;
                    if verify_payment_response.is_err() {
                        error!(
                            "Failed to verify payment {}: {}",
                            id,
                            verify_payment_response.err().unwrap().message
                        );
                    } else {
                        let verify_payment_response = verify_payment_response.unwrap();

                        match verify_payment_response {
                            VerifyPaymentResponse::NeedMore { .. } => {
                                trace!("Need more payment data for payment id {}", id);
                            }
                            VerifyPaymentResponse::Paid { .. } => {
                                info!("Payment {} paid", id);
                                payments_paid_count += 1;
                                service.apply_paid_order(id.clone(), now_in_ns).await;
                            }
                            VerifyPaymentResponse::PaymentNotFound => {
                                todo!("Payment not found, clean order");
                            }
                        }
                    }
                }
            } else {
                debug!("No ids to verify");
            }
            STATE.with(|s| {
                let mut store = s.payment_store.borrow_mut();
                store.set_payment_version_synced_up_to(tip_of_ledger.payments_version);
                store.mark_ledger_sync_complete();
            });

            Ok(payments_paid_count)
        }
    }
}

fn get_block_height_synced_up_to() -> Option<u64> {
    STATE.with(|s| s.payment_store.borrow().get_payment_version_synced_up_to())
}

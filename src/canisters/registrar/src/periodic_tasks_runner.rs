use ic_cdk_macros::*;
use log::error;

use crate::payment_sync;

#[heartbeat]
pub fn heartbeat() {
    let future = run_periodic_tasks();
    ic_cdk::spawn(future);
}

pub async fn run_periodic_tasks() {
    let option = payment_sync::sync_transactions().await;
    if let Some(result) = option {
        if let Err(e) = result {
            error!("Error while syncing transactions: {}", e);
        }
    }
}

use ic_cdk::api;

use log::error;

use crate::payment_sync;
use crate::service::RegistrarService;

pub async fn run_periodic_tasks() {
    // icnaming_ledger is offline, so there is no need to sync it
    // let option = payment_sync::sync_transactions().await;
    // if let Some(result) = option {
    //     if let Err(e) = result {
    //         error!("Error while syncing transactions: {}", e);
    //     }
    // }

    let service = RegistrarService::new();
    let now = api::time();
    let _result = service.cancel_expired_orders(now);
}

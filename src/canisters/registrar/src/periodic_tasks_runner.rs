use common::TimeInNs;
use ic_cdk::api;

use crate::service::RegistrarService;
use crate::token_service::TokenService;

pub async fn run_periodic_tasks() {
    let now = api::time();
    {
        let service = TokenService::default();
        let _result = service.retry_refund(TimeInNs(now));
    }
}

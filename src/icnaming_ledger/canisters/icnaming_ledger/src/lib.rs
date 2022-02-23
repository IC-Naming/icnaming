extern crate core;

use std::collections::HashSet;
use std::str::FromStr;

use candid::candid_method;
use candid::CandidType;
use dfn_candid::{candid, candid_one, CandidOne};
use dfn_core::api::{caller, print};
use dfn_core::stable::get;
use dfn_core::{api, api::trap_with, over, over_async, over_init, stable};
use ic_base_types::{CanisterId, PrincipalId};
use ic_types::messages::RawHttpRequestVal::Array;
use ledger_canister::{AccountIdentifier, BlockHeight, Subaccount};
use serde::{Deserialize, Serialize};

use crate::canisters::ledger::send;
use crate::constants::QUOTA_ORDER_RECEIVE_SUBACCOUNT_FIRST_BYTE;
use crate::named_canister_ids::{
    ensure_current_canister_id_match, get_named_get_canister_id, CANISTER_NAME_ICNAMING_LEDGER,
    CANISTER_NAME_REGISTRAR,
};
use crate::payments_store::{
    get_now, AddPaymentRequest, AddPaymentResponse, GetTipOfLedgerRequest, GetTipOfLedgerResponse,
    RefundPaymentRequest, RefundPaymentResponse, Stats, VerifyPaymentRequest,
    VerifyPaymentResponse,
};
use crate::periodic_tasks_runner::run_periodic_tasks;
use crate::state::{StableState, State, STATE};

mod assets;
mod canisters;
mod constants;
mod ledger_sync;
mod metrics_encoder;
mod named_canister_ids;
mod payments_store;
mod periodic_tasks_runner;
mod settings;
mod state;

#[export_name = "canister_init"]
fn main() {
    main_impl();
}

#[candid_method(init)]
fn main_impl() {
    ensure_current_canister_id_match(CANISTER_NAME_ICNAMING_LEDGER);
    STATE.with(|s| {
        let mut s = s.settings.borrow_mut();

        let current_canister_id = api::id();
        let mut subaccount = [0; std::mem::size_of::<Subaccount>()];
        subaccount[0] = QUOTA_ORDER_RECEIVE_SUBACCOUNT_FIRST_BYTE;
        let receive_account_id = AccountIdentifier::new(
            PrincipalId::from(current_canister_id),
            Some(Subaccount(subaccount)),
        );
        s.receiver_icnaming_ledger_account_ids
            .insert(receive_account_id.clone());
        s.current_receiver_icnaming_ledger_account_id = receive_account_id.clone();

        print(format!(
            "receive_account_id: {}",
            receive_account_id.to_string()
        ));
        let registrar_canister_id = get_named_get_canister_id(CANISTER_NAME_REGISTRAR).to_string();
        print(format!("registrar_canister_id: {}", &registrar_canister_id));
        s.allow_caller_ids
            .insert(PrincipalId::from_str(registrar_canister_id.as_str()).unwrap());

        #[cfg(feature = "dev_canister")]
        {
            let local_test_user =
                include_str!("../../../../configs/dev/principal_registrar_admin.in");
            s.allow_caller_ids
                .insert(PrincipalId::from_str(local_test_user).unwrap());
        }

        assert!(s.allow_caller_ids.len() > 0);

        print("init done");
    });
}

#[export_name = "canister_pre_upgrade"]
fn pre_upgrade() {
    STATE.with(|s| {
        let bytes = s.encode();
        stable::set(&bytes);
    });
}

#[export_name = "canister_post_upgrade"]
fn post_upgrade() {
    ensure_current_canister_id_match(CANISTER_NAME_ICNAMING_LEDGER);
    STATE.with(|s| {
        let bytes = stable::get();
        let new_state = State::decode(bytes).expect("Decoding stable memory failed");

        s.replace(new_state)
    });
}

#[export_name = "canister_query http_request"]
pub fn http_request() {
    over(candid_one, assets::http_request);
}

/// Returns stats about the canister.
///
/// These stats include things such as the number of accounts registered, the memory usage, the
/// number of neurons created, etc.
#[export_name = "canister_query get_stats"]
pub fn get_stats() {
    over(candid, |()| get_stats_impl());
}

fn get_stats_impl() -> Stats {
    STATE.with(|s| s.payments_store.borrow().get_stats())
}

#[export_name = "canister_update add_payment"]
pub fn add_payment() {
    over(candid_one, add_payment_impl);
}

fn add_payment_impl(request: AddPaymentRequest) -> AddPaymentResponse {
    STATE.with(|s| {
        let mut s = s.payments_store.borrow_mut();
        let now = get_now();
        let caller = caller();
        s.add_payment(request, caller, now)
    })
}

#[export_name = "canister_query verify_payment"]
pub fn verify_payment() {
    over(candid_one, verify_payment_impl);
}

fn verify_payment_impl(request: VerifyPaymentRequest) -> VerifyPaymentResponse {
    STATE.with(|s| {
        let s = s.payments_store.borrow();
        s.verify_payment(request)
    })
}

#[export_name = "canister_update refund_payment"]
pub fn refund_payment() {
    over_async(candid_one, refund_payment_impl);
}

async fn refund_payment_impl(request: RefundPaymentRequest) -> RefundPaymentResponse {
    let now = get_now();
    let ready_to_refund_result = STATE.with(|s| {
        let mut s = s.payments_store.borrow_mut();
        let caller = caller();
        s.ready_to_refund(&request, caller, &now)
    });
    if ready_to_refund_result.is_ok() {
        let args = ready_to_refund_result.unwrap();
        let refunded_amount = args.amount;
        let send_result = send(args).await;

        STATE.with(|s| {
            let mut s = s.payments_store.borrow_mut();
            if send_result.is_ok() {
                s.post_refund_send(&request);
                print(format!("refunded {}", refunded_amount));
                RefundPaymentResponse::Refunded { refunded_amount }
            } else {
                s.set_refund_failed(&request);
                print(format!("refund failed"));
                RefundPaymentResponse::RefundFailed
            }
        })
    } else {
        ready_to_refund_result.err().unwrap()
    }
}

#[export_name = "canister_query get_tip_of_ledger"]
pub fn get_tip_of_ledger() {
    over(candid_one, get_tip_of_ledger_impl);
}

pub fn get_tip_of_ledger_impl(request: GetTipOfLedgerRequest) -> GetTipOfLedgerResponse {
    STATE.with(|s| s.payments_store.borrow().get_tip_of_ledger(request))
}

/// Executes on every block height and is used to run background processes.
///
/// These background processes include:
/// - Sync transactions from the ledger
/// - Process any queued 'multi-part' actions (eg. staking a neuron or topping up a canister)
/// - Prune old transactions if memory usage is too high
#[export_name = "canister_heartbeat"]
pub fn canister_heartbeat() {
    let future = run_periodic_tasks();

    dfn_core::api::futures::spawn(future);
}

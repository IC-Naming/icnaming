use std::cell::RefCell;
use std::sync::Once;

use candid::{decode_args, encode_args};
use ic_cdk::{api, storage};
use ic_cdk_macros::*;
use log::info;

use common::ic_logger::ICLogger;
use common::named_canister_ids::CANISTER_NAME_REGISTRAR;
use common::named_canister_ids::{
    ensure_current_canister_id_match, CANISTER_NAME_REGISTRAR_CONTROL_GATEWAY,
};
use common::state::StableState;

use crate::build_gen::ACCEPTABLE_HASHES;
use crate::name_assignment_store::NameAssignmentStore;
use crate::quota_import_store::QuotaImportStore;

thread_local! {
    pub static STATE : State = State::default();
    pub static MERTRICS_COUNTER: RefCell<MetricsCounter> = RefCell::new(MetricsCounter::default());
}

#[derive(Default)]
pub struct MetricsCounter {
    pub last_xdr_permyriad_per_icp: u64,
    pub last_timestamp_seconds_xdr_permyriad_per_icp: u64,
    pub name_order_placed_count: u64,
    pub name_order_paid_count: u64,
    pub name_order_cancelled_count: u64,
    pub new_registered_name_count: u64,
}

#[derive(Default)]
pub struct State {
    pub quota_import_store: RefCell<QuotaImportStore>,
    pub name_assignment_store: RefCell<NameAssignmentStore>,
}

impl State {
    pub fn replace(&self, new_state: State) {
        self.quota_import_store
            .replace(new_state.quota_import_store.take());
        self.name_assignment_store
            .replace(new_state.name_assignment_store.take());
    }
}

impl StableState for State {
    fn encode(&self) -> Vec<u8> {
        encode_args((
            self.quota_import_store.borrow().encode(),
            self.name_assignment_store.borrow().encode(),
        ))
        .unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let (quota_import_store_bytes, name_assignment_store_bytes) = decode_args(&bytes).unwrap();

        Ok(State {
            quota_import_store: RefCell::new(QuotaImportStore::decode(quota_import_store_bytes)?),
            name_assignment_store: RefCell::new(NameAssignmentStore::decode(
                name_assignment_store_bytes,
            )?),
        })
    }
}

static INIT: Once = Once::new();

pub(crate) fn canister_module_init() {
    INIT.call_once(|| {
        ICLogger::init();
    });
    ensure_current_canister_id_match(CANISTER_NAME_REGISTRAR_CONTROL_GATEWAY);

    STATE.with(|s| {
        let mut store = s.quota_import_store.borrow_mut();
        let hashes = ACCEPTABLE_HASHES
            .iter()
            .map(|h| {
                // hex to bytes
                let bytes = hex::decode(h).unwrap();
                bytes
            })
            .collect();
        store.add_acceptable_file_hash(hashes);
    })
}

#[init]
fn init_function() {
    canister_module_init();
}

#[pre_upgrade]
fn pre_upgrade() {
    STATE.with(|s| {
        let bytes = s.encode();
        match storage::stable_save((&bytes,)) {
            Ok(_) => {
                info!("Saved state before upgrade");
                ()
            }
            Err(e) => api::trap(format!("Failed to save state before upgrade: {:?}", e).as_str()),
        };
    });
}

#[post_upgrade]
fn post_upgrade() {
    STATE.with(|s| match storage::stable_restore::<(Vec<u8>,)>() {
        Ok(bytes) => {
            let new_state = State::decode(bytes.0).expect("Decoding stable memory failed");

            s.replace(new_state);
            canister_module_init();
            info!("Loaded state after upgrade");
        }
        Err(e) => api::trap(format!("Failed to restored state after upgrade: {:?}", e).as_str()),
    });
}

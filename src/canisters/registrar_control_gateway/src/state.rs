use candid::{CandidType, Deserialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Once;

use candid::{candid_method, decode_args, encode_args, Principal};
use ic_cdk::{api, storage};
use ic_cdk_macros::*;
use log::info;

use common::ic_logger::ICLogger;

use common::named_canister_ids::{
    ensure_current_canister_id_match, update_dev_named_canister_ids, CanisterNames,
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
    STATE.with(|s| {
        let mut store = s.quota_import_store.borrow_mut();

        // add acceptable hashes to the store
        let hashes = ACCEPTABLE_HASHES
            .iter()
            .map(|h| {
                // hex to bytes
                let bytes = hex::decode(h).unwrap();
                bytes
            })
            .collect();
        store.add_acceptable_file_hash(hashes);

        // add imported file hashes to the store
        let imported_hashes: &[&str] = &[
            "af7619170a528b2ef642224d133297ce3756da745fa4cd84075b59f224e7ab9b",
            "64e72c990a42af6aaf4def6d20b04b827bc302c695efff6d101d39576a6e0232",
            "fdcbd2e084ffc0ad0211bdffa818f3a2d3b70e4652742239e94d6f79c484696e",
            "1edff629db5e2430de2113c240246024b7adda905a47aaba99ec8d9275f16678",
            "20f321798a9a3c5e631f773fcd2ce9e2c214464760fd97f060fdb7593dc8b4cb",
        ];
        let hashes_vec: Vec<Vec<u8>> = imported_hashes
            .iter()
            .map(|h| {
                // hex to bytes
                let bytes = hex::decode(h).unwrap();
                bytes
            })
            .collect();
        for hash in hashes_vec {
            store.add_imported_file_hash(hash);
        }
    })
}

fn guard_func() -> Result<(), String> {
    INIT.call_once(|| {
        ICLogger::init("registrar_control_gateway");
    });
    ensure_current_canister_id_match(CanisterNames::RegistrarControlGateway)
}

#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    dev_named_canister_ids: HashMap<CanisterNames, Principal>,
}

#[init]
#[candid_method(init)]
#[cfg(feature = "dev_env")]
fn init_function(args: Option<InitArgs>) {
    info!("init function called");
    if let Some(args) = args {
        update_dev_named_canister_ids(&args.dev_named_canister_ids);
    }
    canister_module_init();
    guard_func().unwrap();
}

#[init]
#[candid_method(init)]
#[cfg(not(feature = "dev_env"))]
fn init_function() {
    info!("init function called");
    canister_module_init();
    guard_func().unwrap();
}

#[pre_upgrade(guard = "guard_func")]
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

#[post_upgrade(guard = "guard_func")]
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

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::sync::Once;

use candid::{candid_method, decode_args, encode_args, Principal};
use ic_cdk::{api, storage};
use ic_cdk_macros::*;
use log::info;

use crate::balance_store::BalanceStore;
use candid::{CandidType, Deserialize};
use common::ic_logger::ICLogger;
use common::named_canister_ids::{
    ensure_current_canister_id_match, update_dev_named_canister_ids, CanisterNames,
};
use common::state::StableState;

use crate::name_locker::NameLocker;
use crate::name_order_store::NameOrderStore;
use crate::payment_store::PaymentStore;
use crate::quota_import_store::QuotaImportStore;
use crate::registration_approval_store::RegistrationApprovalStore;
use crate::registration_store::{Registration, RegistrationStore};
use crate::settings::Settings;
use crate::user_quota_store::UserQuotaStore;

thread_local! {
    pub static STATE : State = State::default();
    pub static MERTRICS_COUNTER: RefCell<MetricsCounter> = RefCell::new(MetricsCounter::default());
    pub static NAME_LOCKER: RefCell<NameLocker> = RefCell::new(NameLocker::new());
}

#[derive(Default)]
pub struct MetricsCounter {
    pub last_xdr_permyriad_per_icp: u64,
    pub last_timestamp_seconds_xdr_permyriad_per_icp: u64,
    pub last_registrations: VecDeque<Registration>,
    // TODO remove counters below since useless
    pub name_order_placed_count: u64,
    pub name_order_paid_count: u64,
    pub name_order_cancelled_count: u64,
    pub new_registered_name_count: u64,
}

impl MetricsCounter {
    pub fn push_registration(&mut self, registration: Registration) {
        self.last_registrations.push_front(registration);
        if self.last_registrations.len() > 50 {
            self.last_registrations.pop_back();
        }
    }
}

#[derive(Default)]
pub struct State {
    // NOTE: When adding new persistent fields here, ensure that these fields
    // are being persisted in the `replace` method below.
    pub name_order_store: RefCell<NameOrderStore>,
    pub payment_store: RefCell<PaymentStore>,
    pub settings: RefCell<Settings>,
    pub user_quota_store: RefCell<UserQuotaStore>,
    pub registration_store: RefCell<RegistrationStore>,
    pub quota_import_store: RefCell<QuotaImportStore>,
    pub registration_approval_store: RefCell<RegistrationApprovalStore>,
    pub balance_store: RefCell<BalanceStore>,
}

impl State {
    pub fn replace(&self, new_state: State) {
        self.settings.replace(new_state.settings.take());
        self.registration_store
            .replace(new_state.registration_store.take());
        self.name_order_store
            .replace(new_state.name_order_store.take());
        self.payment_store.replace(new_state.payment_store.take());
        self.user_quota_store
            .replace(new_state.user_quota_store.take());
        self.quota_import_store
            .replace(new_state.quota_import_store.take());
        self.registration_approval_store
            .replace(new_state.registration_approval_store.take());
        self.balance_store.replace(new_state.balance_store.take());
    }
}

pub type EncodedState = (
    Option<Vec<u8>>,
    Option<Vec<u8>>,
    Option<Vec<u8>>,
    Option<Vec<u8>>,
    Option<Vec<u8>>,
    Option<Vec<u8>>,
    Option<Vec<u8>>,
    Option<Vec<u8>>,
);

impl StableState for State {
    fn encode(&self) -> Vec<u8> {
        encode_args((
            self.settings.borrow().encode(),
            self.registration_store.borrow().encode(),
            self.name_order_store.borrow().encode(),
            self.payment_store.borrow().encode(),
            self.user_quota_store.borrow().encode(),
            self.quota_import_store.borrow().encode(),
            self.registration_approval_store.borrow().encode(),
            self.balance_store.borrow().encode(),
        ))
        .unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let (
            settings_bytes,
            registration_store_bytes,
            name_order_store_bytes,
            payment_store_bytes,
            user_quota_store_bytes,
            quota_import_store_bytes,
            registration_approval_store_bytes,
            balance_store_bytes,
        ): EncodedState = decode_args(&bytes).unwrap();

        return Ok(State {
            name_order_store: decode_or_default::<NameOrderStore>(name_order_store_bytes)?,
            payment_store: decode_or_default::<PaymentStore>(payment_store_bytes)?,
            settings: decode_or_default::<Settings>(settings_bytes)?,
            user_quota_store: decode_or_default::<UserQuotaStore>(user_quota_store_bytes)?,
            registration_store: decode_or_default::<RegistrationStore>(registration_store_bytes)?,
            quota_import_store: decode_or_default::<QuotaImportStore>(quota_import_store_bytes)?,
            registration_approval_store: decode_or_default::<RegistrationApprovalStore>(
                registration_approval_store_bytes,
            )?,
            balance_store: decode_or_default::<BalanceStore>(balance_store_bytes)?,
        });

        fn decode_or_default<T>(bytes: Option<Vec<u8>>) -> Result<RefCell<T>, String>
        where
            T: Default + StableState,
        {
            let inner = if let Some(bytes) = bytes {
                T::decode(bytes)?
            } else {
                T::default()
            };
            Ok(RefCell::new(inner))
        }
    }
}

static INIT: Once = Once::new();

fn guard_func() -> Result<(), String> {
    INIT.call_once(|| {
        ICLogger::init("registrar");
    });
    ensure_current_canister_id_match(CanisterNames::Registrar)
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

    guard_func().unwrap();
}

#[init]
#[candid_method(init)]
#[cfg(not(feature = "dev_env"))]
fn init_function() {
    info!("init function called");
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
            info!("Loaded state after upgrade");
        }
        Err(e) => api::trap(format!("Failed to restored state after upgrade: {:?}", e).as_str()),
    });
}

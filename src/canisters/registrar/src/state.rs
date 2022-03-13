use std::cell::RefCell;
use std::sync::Once;

use candid::{decode_args, encode_args};
use ic_cdk::{api, storage};
use ic_cdk_macros::*;
use log::info;

use common::ic_logger::ICLogger;
use common::named_canister_ids::ensure_current_canister_id_match;
use common::named_canister_ids::CANISTER_NAME_REGISTRAR;
use common::state::StableState;

use crate::name_order_store::NameOrderStore;
use crate::payment_store::PaymentStore;
use crate::quota_import_store::QuotaImportStore;
use crate::quota_order_store::QuotaOrderStore;
use crate::registration_approval_store::RegistrationApprovalStore;
use crate::registration_store::RegistrationStore;
use crate::settings::Settings;
use crate::user_quota_store::UserQuotaStore;

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
    // NOTE: When adding new persistent fields here, ensure that these fields
    // are being persisted in the `replace` method below.
    pub name_order_store: RefCell<NameOrderStore>,
    pub payment_store: RefCell<PaymentStore>,
    pub settings: RefCell<Settings>,
    pub user_quota_store: RefCell<UserQuotaStore>,
    pub quota_order_store: RefCell<QuotaOrderStore>,
    pub registration_store: RefCell<RegistrationStore>,
    pub quota_import_store: RefCell<QuotaImportStore>,
    pub registration_approval_store: RefCell<RegistrationApprovalStore>,
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
        self.quota_order_store
            .replace(new_state.quota_order_store.take());
        self.quota_import_store
            .replace(new_state.quota_import_store.take());
        self.registration_approval_store
            .replace(new_state.registration_approval_store.take());
    }
}

impl StableState for State {
    fn encode(&self) -> Vec<u8> {
        encode_args((
            self.settings.borrow().encode(),
            self.registration_store.borrow().encode(),
            self.name_order_store.borrow().encode(),
            self.payment_store.borrow().encode(),
            self.user_quota_store.borrow().encode(),
            self.quota_order_store.borrow().encode(),
            self.quota_import_store.borrow().encode(),
            self.registration_approval_store.borrow().encode(),
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
            quota_order_store_bytes,
            quota_import_store_bytes,
            registration_approval_store_bytes,
        ): (
            Vec<u8>,
            Vec<u8>,
            Vec<u8>,
            Vec<u8>,
            Vec<u8>,
            Vec<u8>,
            Vec<u8>,
            Option<Vec<u8>>,
        ) = decode_args(&bytes).unwrap();

        let registration_approval_store = if let Some(bytes) = registration_approval_store_bytes {
            RegistrationApprovalStore::decode(bytes)?
        } else {
            RegistrationApprovalStore::default()
        };

        Ok(State {
            name_order_store: RefCell::new(NameOrderStore::decode(name_order_store_bytes)?),
            payment_store: RefCell::new(PaymentStore::decode(payment_store_bytes)?),
            settings: RefCell::new(Settings::decode(settings_bytes)?),
            user_quota_store: RefCell::new(UserQuotaStore::decode(user_quota_store_bytes)?),
            quota_order_store: RefCell::new(QuotaOrderStore::decode(quota_order_store_bytes)?),
            registration_store: RefCell::new(RegistrationStore::decode(registration_store_bytes)?),
            quota_import_store: RefCell::new(QuotaImportStore::decode(quota_import_store_bytes)?),
            registration_approval_store: RefCell::new(registration_approval_store),
        })
    }
}

static INIT: Once = Once::new();

pub(crate) fn canister_module_init() {
    INIT.call_once(|| {
        ICLogger::init();
    });
    ensure_current_canister_id_match(CANISTER_NAME_REGISTRAR);
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

use std::borrow::Borrow;
use std::cell::RefCell;
use std::str::FromStr;

use dfn_candid::Candid;
use ic_base_types::CanisterId;
use on_wire::{FromWire, IntoWire};

use crate::payments_store::PaymentsStore;
use crate::settings::Settings;

#[derive(Default)]
pub struct State {
    // NOTE: When adding new persistent fields here, ensure that these fields
    // are being persisted in the `replace` method below.
    pub payments_store: RefCell<PaymentsStore>,
    pub settings: RefCell<Settings>,
}

impl State {
    pub fn replace(&self, new_state: State) {
        self.payments_store.replace(new_state.payments_store.take());
        self.settings.replace(new_state.settings.take());
    }
}

pub trait StableState: Sized {
    fn encode(&self) -> Vec<u8>;
    fn decode(bytes: Vec<u8>) -> Result<Self, String>;
}

thread_local! {
    pub static STATE: State = State::default();
}

impl StableState for State {
    fn encode(&self) -> Vec<u8> {
        Candid((
            self.payments_store.borrow().encode(),
            self.settings.borrow().encode(),
        ))
        .into_bytes()
        .unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let (account_store_bytes, icnaming_ledger_settings_bytes) =
            Candid::from_bytes(bytes).map(|c| c.0)?;

        Ok(State {
            payments_store: RefCell::new(PaymentsStore::decode(account_store_bytes)?),
            settings: RefCell::new(Settings::decode(icnaming_ledger_settings_bytes)?),
        })
    }
}

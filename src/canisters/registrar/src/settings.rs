use candid::{decode_args, encode_args};

use crate::state::STATE;
use common::constants::{DEFAULT_MAX_REGISTRATION_YEAR, DEFAULT_MIN_REGISTRATION_YEAR};
use common::errors::{ICNSError, ICNSResult};
use common::state::StableState;

pub struct Settings {
    pub min_year: u32,
    pub max_year: u32,
    pub maintaining_time: Option<u64>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings::new()
    }
}

impl StableState for Settings {
    fn encode(&self) -> Vec<u8> {
        encode_args((&self.min_year, &self.max_year, &self.maintaining_time)).unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let (min_year, max_year, maintaining_time): (u32, u32, Option<u64>) =
            decode_args(&bytes).unwrap();

        Ok(Settings {
            min_year,
            max_year,
            maintaining_time,
        })
    }
}

impl Settings {
    pub fn new() -> Self {
        Self {
            min_year: DEFAULT_MIN_REGISTRATION_YEAR,
            max_year: DEFAULT_MAX_REGISTRATION_YEAR,
            maintaining_time: None,
        }
    }

    pub fn is_year_valid(&self, year: u32) -> bool {
        year >= self.min_year && year < self.max_year
    }

    pub fn set_maintaining_time(&mut self, time: u64) {
        if time == 0u64 {
            self.maintaining_time = None;
        } else {
            self.maintaining_time = Some(time);
        }
    }

    pub fn is_maintaining(&self, now: u64) -> bool {
        self.maintaining_time
            .map(|time| now > time)
            .unwrap_or(false)
    }
}

pub fn check_system_is_maintaining(now: u64) -> ICNSResult<()> {
    let is_maintaining = STATE.with(|state| {
        let settings = state.settings.borrow();
        settings.is_maintaining(now)
    });
    if is_maintaining {
        Err(ICNSError::SystemMaintaining)
    } else {
        Ok(())
    }
}

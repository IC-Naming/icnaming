use candid::{decode_args, encode_args};

use common::constants::{DEFAULT_MAX_REGISTRATION_YEAR, DEFAULT_MIN_REGISTRATION_YEAR};
use common::state::StableState;

pub struct Settings {
    pub min_year: u32,
    pub max_year: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings::new()
    }
}

impl StableState for Settings {
    fn encode(&self) -> Vec<u8> {
        encode_args((&self.min_year, &self.max_year)).unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let (min_year, max_year): (u32, u32) = decode_args(&bytes).unwrap();

        Ok(Settings { min_year, max_year })
    }
}

impl Settings {
    pub fn new() -> Self {
        Self {
            min_year: DEFAULT_MIN_REGISTRATION_YEAR,
            max_year: DEFAULT_MAX_REGISTRATION_YEAR,
        }
    }
    pub(crate) fn get_min_year(&self) -> u32 {
        self.min_year
    }
    pub(crate) fn get_max_year(&self) -> u32 {
        self.max_year
    }

    pub fn is_year_valid(&self, year: u32) -> bool {
        year >= self.min_year && year < self.max_year
    }
}

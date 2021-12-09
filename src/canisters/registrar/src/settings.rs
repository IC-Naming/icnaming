use common::constants::{DEFAULT_MAX_REGISTRATION_YEAR, DEFAULT_MIN_REGISTRATION_YEAR};

pub struct Settings {
    pub min_year: u64,
    pub max_year: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Settings::new()
    }
}

impl Settings {
    pub fn new() -> Self {
        Self {
            min_year: DEFAULT_MIN_REGISTRATION_YEAR,
            max_year: DEFAULT_MAX_REGISTRATION_YEAR,
        }
    }
    pub(crate) fn get_min_year(&self) -> u64 {
        self.min_year
    }
    pub(crate) fn get_max_year(&self) -> u64 {
        self.max_year
    }

    pub fn is_year_valid(&self, year: u64) -> bool {
        year >= self.min_year && year < self.max_year
    }
}

use candid::{CandidType, Deserialize, Principal};

use crate::models::Registration;
use crate::settings::Settings;

#[derive(Eq, PartialEq, Debug, Clone, CandidType, Deserialize)]
pub(crate) struct RegistrationStable {
    owner: Principal,
    name: String,
    expired_at: u64,
    created_at: u64,
}

// Registration -> RegistrationStable
impl From<&Registration> for RegistrationStable {
    fn from(registration: &Registration) -> Self {
        RegistrationStable {
            owner: registration.get_owner(),
            name: registration.get_name(),
            expired_at: registration.get_expired_at(),
            created_at: registration.get_created_at(),
        }
    }
}

// RegistrationStable -> Registration
impl From<&RegistrationStable> for Registration {
    fn from(registration_stable: &RegistrationStable) -> Self {
        Registration::new(
            registration_stable.owner.clone(),
            registration_stable.name.clone(),
            registration_stable.expired_at,
            registration_stable.created_at,
        )
    }
}

#[derive(Debug, Clone, CandidType, Deserialize)]
pub(crate) struct SettingsStable {
    pub min_year: u64,
    pub max_year: u64,
}

// &Settings -> SettingsStable
impl From<&Settings> for SettingsStable {
    fn from(settings: &Settings) -> Self {
        SettingsStable {
            min_year: settings.get_min_year(),
            max_year: settings.get_max_year(),
        }
    }
}

// SettingsStable -> Settings
impl From<&SettingsStable> for Settings {
    fn from(settings_stable: &SettingsStable) -> Self {
        Settings {
            min_year: settings_stable.min_year,
            max_year: settings_stable.max_year,
        }
    }
}

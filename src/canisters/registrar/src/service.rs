use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use candid::Principal;

use common::canister_api::ic_impl::RegistryApi;
use common::canister_api::IRegistryApi;
use common::constants::{CANISTER_NAME_RESOLVER, DEFAULT_TTL, TOP_LABEL};
use common::dto::{GetPageInput, GetPageOutput};
use common::errors::ICNSError::RemoteError;
use common::errors::{ICNSError, ICNSResult};
use common::ic_api::wrapper::ICStaticApi;
use common::ic_api::IClock;
use common::naming::{normalize_name, NameParseResult};
use common::state::get_principal;

use crate::models::*;
use crate::state::{REGISTRATIONS, SETTINGS};

#[cfg(test)]
mod tests;

pub struct RegistrarService {
    pub registry_api: Arc<dyn IRegistryApi>,
    pub clock: Arc<dyn IClock>,
}

impl Debug for RegistrarService {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, stringify!(RegistrarService))
    }
}

impl Default for RegistrarService {
    fn default() -> Self {
        RegistrarService::new()
    }
}

impl RegistrarService {
    pub fn new() -> RegistrarService {
        RegistrarService {
            registry_api: Arc::new(RegistryApi),
            clock: Arc::new(ICStaticApi::new()),
        }
    }

    pub(crate) fn get_names(
        &self,
        owner: &Principal,
        input: &GetPageInput,
    ) -> ICNSResult<GetPageOutput<RegistrationDto>> {
        input.validate()?;
        // validate owner
        if *owner == Principal::anonymous() {
            return Err(ICNSError::InvalidOwner);
        }
        let items = REGISTRATIONS.with(|registrations| {
            registrations
                .borrow()
                .values()
                .filter(|registration| registration.is_owner(owner))
                .skip(input.offset)
                .take(input.limit)
                .map(|registration| registration.into())
                .collect()
        });

        Ok(GetPageOutput::new(items))
    }

    pub(crate) fn get_details(&self, name: &str) -> ICNSResult<RegistrationDetails> {
        let name = normalize_name(name);
        REGISTRATIONS.with(|registrations| {
            let registrations = registrations.borrow();
            let registration = registrations.get(&name);
            if registration.is_none() {
                return Err(ICNSError::RegistrationNotFound);
            }
            Ok(RegistrationDetails::from(registration.unwrap()))
        })
    }

    pub(crate) fn get_owner(&self, name: &str) -> ICNSResult<Principal> {
        let name = normalize_name(name);
        REGISTRATIONS.with(|registrations| {
            let registrations = registrations.borrow();
            let registration = registrations.get(&name);
            if registration.is_none() {
                return Err(ICNSError::RegistrationNotFound);
            }
            Ok(registration.unwrap().get_owner().to_owned())
        })
    }

    pub(crate) fn get_name_expires(&self, name: &str) -> ICNSResult<u64> {
        let name = self.normalize_name(&name);
        REGISTRATIONS.with(|registrations| {
            let registrations = registrations.borrow();
            let registration = registrations.get(&name);
            if let Some(registration) = registration {
                return Ok(registration.get_expired_at());
            }
            Err(ICNSError::RegistrationNotFound)
        })
    }
    pub fn normalize_name(&self, name: &str) -> String {
        normalize_name(name)
    }
    pub fn validate_name(&self, name: &str) -> Result<NameParseResult, String> {
        let result = NameParseResult::parse(name);
        if result.get_level_count() != 2 {
            return Err("it must be second level name".to_string());
        }
        if result.get_top_level().unwrap() != TOP_LABEL {
            return Err(format!("top level of name must be {}", TOP_LABEL));
        }
        let first = result.get_current_level().unwrap();
        if first.len() > 63 {
            return Err("second level name must be less than 64 characters".to_string());
        }
        if first.len() < 4 {
            return Err("second level name must be more than 3 characters".to_string());
        }
        if !first.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err("name must be alphanumeric or -".to_string());
        }
        return Ok(result);
    }

    pub async fn register(
        &mut self,
        name: &str,
        owner: &Principal,
        years: u64,
        now_in_ms: u64,
    ) -> ICNSResult<bool> {
        let name = self.normalize_name(&name);
        let result = self.validate_name(&name);
        if result.is_err() {
            return Err(ICNSError::InvalidName {
                reason: result.err().unwrap().to_string(),
            });
        }
        if *owner == Principal::anonymous() {
            return Err(ICNSError::InvalidOwner);
        }
        REGISTRATIONS.with(|registrations| {
            let registrations = registrations.borrow();
            if registrations.contains_key(&name) {
                return Err(ICNSError::RegistrationHasBeenTaken);
            }
            Ok(())
        })?;
        // validate year
        SETTINGS.with(|settings| {
            let settings = settings.borrow();
            if !settings.is_year_valid(years) {
                return Err(ICNSError::YearsRangeError {
                    min: settings.min_year,
                    max: settings.max_year,
                });
            }
            Ok(())
        })?;

        // TODO adjusts to date format w/o seconds
        // keep date for now_in_ms
        let expired_at = now_in_ms + year_to_ms(years);
        let resolver = get_principal(CANISTER_NAME_RESOLVER).unwrap();
        let registration = Registration::new(owner.clone(), name.clone(), expired_at, now_in_ms);
        let api_result = self
            .registry_api
            .set_subdomain_owner(
                result.unwrap().get_current_level().unwrap().clone(),
                TOP_LABEL.to_string(),
                owner.clone(),
                DEFAULT_TTL,
                resolver,
            )
            .await;
        if api_result.is_ok() {
            REGISTRATIONS.with(|registrations| {
                let mut registrations = registrations.borrow_mut();
                registrations.insert(name.clone(), registration);
            });
            Ok(true)
        } else {
            Err(RemoteError(api_result.err().unwrap()))
        }
    }

    pub fn available(&self, name: &str) -> ICNSResult<bool> {
        let name = self.normalize_name(name);
        let result = self.validate_name(&name);
        if result.is_err() {
            return Err(ICNSError::InvalidName {
                reason: result.err().unwrap().to_string(),
            });
        }
        REGISTRATIONS.with(|registrations| {
            let registrations = registrations.borrow();
            if registrations.contains_key(&name) {
                return Err(ICNSError::RegistrationHasBeenTaken);
            }
            Ok(true)
        })
    }

    pub fn clean_expired(&mut self, _now_in_ms: u64) -> ICNSResult<()> {
        todo!("clean up")
    }
}

fn year_to_ms(years: u64) -> u64 {
    years * 365 * 24 * 60 * 60 * 1000
}

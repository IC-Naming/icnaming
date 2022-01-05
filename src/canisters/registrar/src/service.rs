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
use common::state::{get_principal, is_owner, NAMED_PRINCIPALS};

use crate::models::*;
use crate::state::{REGISTRATIONS, SETTINGS, USER_QUOTA_MANAGER};

#[cfg(test)]
mod tests;

pub struct RegistrarService {
    pub registry_api: Arc<dyn IRegistryApi>,
    pub clock: Arc<dyn IClock>,
}

impl RegistrarService {}

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

    pub fn validate_quota(
        &self,
        name: &NameParseResult,
        owner: &Principal,
        quota_type: &QuotaType,
    ) -> Result<(), String> {
        let first = name.get_current_level().unwrap();
        match quota_type {
            QuotaType::LenEq(len) => {
                if first.chars().count() != len.clone() as usize {
                    return Err(format!("Name must be exactly {} characters long", len));
                }
            }
            QuotaType::LenGte(len) => {
                if first.chars().count() < len.clone() as usize {
                    return Err(format!("Name must be at least {} characters long", len));
                }
            }
        }
        let result = USER_QUOTA_MANAGER.with(|user_quota_manager| {
            let user_quota_manager = user_quota_manager.borrow();
            let quota = user_quota_manager
                .get_quota(owner, &quota_type)
                .unwrap_or(0);
            if quota == 0 {
                return Err(format!("User has no quota for {}", quota_type));
            }
            return Ok(());
        });
        result
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
        quota_owner: &Principal,
        quota_type: QuotaType,
    ) -> ICNSResult<bool> {
        let name = self.normalize_name(&name);

        // validate name
        let name_result = self.validate_name(&name);
        if name_result.is_err() {
            return Err(ICNSError::InvalidName {
                reason: name_result.err().unwrap().to_string(),
            });
        }
        let name_result = name_result.unwrap();

        // validate user
        if *owner == Principal::anonymous() {
            return Err(ICNSError::InvalidOwner);
        }

        // validate quota_owner
        if *quota_owner == Principal::anonymous() {
            return Err(ICNSError::InvalidOwner);
        }

        // validate quota
        let quota_result = self.validate_quota(&name_result, quota_owner, &quota_type);
        if quota_result.is_err() {
            return Err(ICNSError::InvalidName {
                reason: quota_result.err().unwrap().to_string(),
            });
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

        // update quota before await in case of concurrent register
        USER_QUOTA_MANAGER.with(|user_quota_manager| {
            let mut user_quota_manager = user_quota_manager.borrow_mut();
            let result = user_quota_manager.sub_quota(quota_owner, &quota_type, 1);
            assert_eq!(result, true);
        });

        // TODO adjusts to date format w/o seconds
        // keep date for now_in_ms
        let expired_at = now_in_ms + year_to_ms(years);
        let resolver = get_principal(CANISTER_NAME_RESOLVER).unwrap();
        let registration = Registration::new(owner.clone(), name.clone(), expired_at, now_in_ms);
        let api_result = self
            .registry_api
            .set_subdomain_owner(
                name_result.get_current_level().unwrap().clone(),
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
            // rollback quota
            USER_QUOTA_MANAGER.with(|user_quota_manager| {
                let mut user_quota_manager = user_quota_manager.borrow_mut();
                user_quota_manager.add_quota(quota_owner.clone(), quota_type, 1);
            });
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

    pub fn add_quota(
        &mut self,
        caller: &Principal,
        quota_owner: Principal,
        quota_type: QuotaType,
        diff: u32,
    ) -> ICNSResult<bool> {
        if !is_owner(caller) {
            return Err(ICNSError::OwnerOnly);
        };
        USER_QUOTA_MANAGER.with(|user_quota_manager| {
            let mut user_quota_manager = user_quota_manager.borrow_mut();
            user_quota_manager.add_quota(quota_owner, quota_type, diff);
        });
        Ok(true)
    }

    pub fn sub_quota(
        &mut self,
        caller: &Principal,
        quota_owner: Principal,
        quota_type: QuotaType,
        diff: u32,
    ) -> ICNSResult<bool> {
        if !is_owner(caller) {
            return Err(ICNSError::OwnerOnly);
        };
        USER_QUOTA_MANAGER.with(|user_quota_manager| {
            let mut user_quota_manager = user_quota_manager.borrow_mut();
            user_quota_manager.sub_quota(&quota_owner, &quota_type, diff);
        });
        Ok(true)
    }

    pub fn get_quota(
        &self,
        caller: &Principal,
        quota_owner: Principal,
        quota_type: QuotaType,
    ) -> ICNSResult<u32> {
        if !is_owner(caller) {
            return Err(ICNSError::OwnerOnly);
        };
        USER_QUOTA_MANAGER.with(|user_quota_manager| {
            let user_quota_manager = user_quota_manager.borrow();
            Ok(user_quota_manager
                .get_quota(&quota_owner, &quota_type)
                .unwrap_or(0))
        })
    }
}

fn year_to_ms(years: u64) -> u64 {
    years * 365 * 24 * 60 * 60 * 1000
}

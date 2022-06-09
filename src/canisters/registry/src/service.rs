use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use candid::Principal;

use log::{debug, error, info};

use common::canister_api::ic_impl::ResolverApi;
use common::canister_api::IResolverApi;
use common::constants::{DEFAULT_TTL, MAX_REGISTRY_OPERATOR_COUNT, NAMING_TOP_LABEL};
use common::dto::{GetPageInput, GetPageOutput, IRegistryUsers, RegistryDto, RegistryUsers};
use common::errors::{NamingError, ServiceResult};
use common::named_canister_ids::CanisterNames;

use common::permissions::{must_be_named_canister, must_not_anonymous};

use crate::registry_store::*;
use crate::state::STATE;

#[cfg(test)]
mod tests;

pub struct RegistriesService {
    pub resolver_api: Arc<dyn IResolverApi>,
}

pub fn get_registry<'a>(
    registries: &'a HashMap<String, Registry>,
    name: &str,
) -> ServiceResult<&'a Registry> {
    let registry = registries.get(name);
    if let Some(registry) = registry {
        Ok(registry)
    } else {
        Err(NamingError::RegistryNotFoundError {
            name: name.to_string(),
        })
    }
}

pub fn get_registry_mut<'a>(
    registries: &'a mut HashMap<String, Registry>,
    name: &str,
) -> ServiceResult<&'a mut Registry> {
    let registry = registries.get_mut(name);
    if let Some(registry) = registry {
        Ok(registry)
    } else {
        Err(NamingError::RegistryNotFoundError {
            name: name.to_string(),
        })
    }
}

impl RegistriesService {
    pub fn new() -> Self {
        Self {
            resolver_api: Arc::new(ResolverApi::default()),
        }
    }

    pub fn set_top_icp_name(&mut self, registrar: Principal) -> ServiceResult<bool> {
        STATE.with(|s| {
            let store = s.registry_store.borrow();
            let registries = store.get_registries();
            if registries.contains_key(NAMING_TOP_LABEL) {
                Err(NamingError::TopNameAlreadyExists)
            } else {
                Ok(true)
            }
        })?;

        self.set_top_name(Registry::new(
            NAMING_TOP_LABEL.to_string(),
            registrar,
            DEFAULT_TTL,
            Principal::anonymous(),
        ))
    }

    fn set_top_name(&mut self, registry: Registry) -> ServiceResult<bool> {
        STATE.with(|s| {
            let mut store = s.registry_store.borrow_mut();
            let registries = store.get_registries_mut();
            if !registries.is_empty() {
                return Err(NamingError::TopNameAlreadyExists);
            }
            registries.insert(registry.get_name().to_string(), registry);
            Ok(true)
        })
    }

    pub async fn set_subdomain_owner(
        &mut self,
        label: String,
        parent_name: String,
        owner: Principal,
        sub_owner: Principal,
        ttl: u64,
        resolver: Principal,
    ) -> ServiceResult<RegistryDto> {
        debug!("set_subdomain_owner: label: {}, parent_name: {}, owner: {}, sub_owner: {}, ttl: {}, resolver: {}", label, parent_name, owner, sub_owner, ttl, resolver);

        STATE.with(|s| {
            let store = s.registry_store.borrow();
            let registries = store.get_registries();
            let registry = get_registry(registries, &parent_name)?;
            if !registry.is_owner(&owner) {
                Err(NamingError::PermissionDenied)
            } else {
                Ok(true)
            }
        })?;

        let subdomain_name = format!("{}.{}", label, parent_name);
        // find old registry
        let updated_registry = STATE.with(|s| {
            let mut store = s.registry_store.borrow_mut();
            let registries = store.get_registries_mut();
            let old_registry = registries.get_mut(&subdomain_name);
            if let Some(old_registry) = old_registry {
                info!("old_registry: {:?}", old_registry);
                // update owner of old registry
                old_registry.set_owner(sub_owner);
                old_registry.set_ttl(ttl);
                old_registry.set_resolver(resolver);
                old_registry.clone()
            } else {
                // create new registry
                let new_registry = Registry::new(subdomain_name.clone(), sub_owner, ttl, resolver);
                let subdomain_name = format!("{}.{}", label, parent_name);
                registries.insert(subdomain_name, new_registry.clone());
                new_registry
            }
        });

        let result = self
            .resolver_api
            .ensure_resolver_created(subdomain_name.clone())
            .await;
        info!("ensure_resolver_created: {:?}", result);
        Ok(RegistryDto::from(&updated_registry))
    }

    pub fn check_exist(&self, name: &str) -> bool {
        STATE.with(|s| {
            let store = s.registry_store.borrow();
            let registries = store.get_registries();
            registries.contains_key(name)
        })
    }

    pub fn get_resolver(&self, name: &str) -> ServiceResult<Principal> {
        STATE.with(|s| {
            let store = s.registry_store.borrow();
            let registries = store.get_registries();
            let registry = get_registry(registries, name)?;
            Ok(registry.get_resolver())
        })
    }

    pub fn set_record(
        &mut self,
        caller: &Principal,
        name: &str,
        ttl: u64,
        resolver: &Principal,
    ) -> ServiceResult<bool> {
        STATE.with(|s| {
            let mut store = s.registry_store.borrow_mut();
            let registries = store.get_registries_mut();
            let registry = get_registry_mut(registries, name)?;
            if !registry.can_operate(caller) {
                return Err(NamingError::PermissionDenied);
            }
            registry.set_ttl(ttl);
            registry.set_resolver(*resolver);
            Ok(true)
        })
    }

    pub fn set_resolver(
        &self,
        caller: Principal,
        name: &str,
        resolver: Principal,
    ) -> ServiceResult<bool> {
        STATE.with(|s| {
            let mut store = s.registry_store.borrow_mut();
            let registries = store.get_registries_mut();
            let registry = get_registry_mut(registries, name)?;
            if !registry.can_operate(&caller) {
                return Err(NamingError::PermissionDenied);
            }
            registry.set_resolver(resolver);
            Ok(true)
        })
    }

    pub(crate) fn get_controlled_names(
        &self,
        owner: Principal,
        page: GetPageInput,
    ) -> ServiceResult<GetPageOutput<String>> {
        STATE.with(|s| {
            let store = s.registry_store.borrow();
            let registries = store.get_registries();
            let resolver_names = registries.iter().filter_map(|(name, registry)| {
                if registry.is_owner(&owner) {
                    Some(name.clone())
                } else {
                    None
                }
            });

            let resolver_names = resolver_names
                .into_iter()
                .skip(page.offset)
                .take(page.limit);
            let resolver_names = resolver_names.collect::<Vec<_>>();
            Ok(GetPageOutput {
                items: resolver_names,
            })
        })
    }
    pub fn set_approval(
        &mut self,
        name: &str,
        caller: &Principal,
        operator: &Principal,
    ) -> ServiceResult<bool> {
        STATE.with(|s| {
            let mut store = s.registry_store.borrow_mut();
            let registries = store.get_registries_mut();
            let registry = get_registry_mut(registries, name)?;
            if !registry.is_owner(caller) {
                return Err(NamingError::PermissionDenied);
            }
            if caller == operator {
                return Err(NamingError::OperatorShouldNotBeTheSameToOwner);
            }
            let operator_count = registry.get_operator_count();
            if operator_count + 1 >= MAX_REGISTRY_OPERATOR_COUNT {
                return Err(NamingError::OperatorCountExceeded);
            }
            registry.add_operator(operator);
            Ok(true)
        })
    }

    pub fn remove_approval(
        &mut self,
        name: &str,
        caller: &Principal,
        operator: &Principal,
    ) -> ServiceResult<bool> {
        STATE.with(|s| {
            let mut store = s.registry_store.borrow_mut();
            let registries = store.get_registries_mut();
            let registry = get_registry_mut(registries, name)?;
            if !registry.is_owner(caller) {
                return Err(NamingError::PermissionDenied);
            }
            registry.remove_operator(operator);
            Ok(true)
        })
    }

    pub(crate) fn get_users(&self, name: &str) -> ServiceResult<RegistryUsers> {
        STATE.with(|s| {
            let store = s.registry_store.borrow();
            let registries = store.get_registries();
            let registry = get_registry(registries, name)?;
            Ok(registry.get_users())
        })
    }

    pub(crate) fn get_owner(&self, name: &str) -> ServiceResult<Principal> {
        STATE.with(|s| {
            let store = s.registry_store.borrow();
            let registries = store.get_registries();
            let registry = get_registry(registries, name)?;
            Ok(registry.get_owner().to_owned())
        })
    }

    pub(crate) fn set_owner(
        &self,
        caller: Principal,
        name: &str,
        owner: Principal,
    ) -> ServiceResult<bool> {
        must_not_anonymous(&caller)?;
        must_not_anonymous(&owner)?;
        STATE.with(|s| {
            let mut store = s.registry_store.borrow_mut();
            let registries = store.get_registries_mut();
            let registry = get_registry_mut(registries, name)?;
            let old_owner = registry.get_owner();
            if *old_owner != caller {
                error!("{} is not the owner of {}", caller, name);
                return Err(NamingError::PermissionDenied);
            }

            if *old_owner == owner {
                error!("{} is already the owner of {}", owner, name);
                return Err(NamingError::InvalidOwner);
            }
            registry.set_owner(owner);
            info!("{} is set as the owner of {}", owner, name);
            Ok(true)
        })
    }

    pub(crate) fn get_ttl(&self, name: &str) -> ServiceResult<u64> {
        STATE.with(|s| {
            let store = s.registry_store.borrow();
            let registries = store.get_registries();
            let registry = get_registry(registries, name)?;
            Ok(registry.get_ttl())
        })
    }

    pub(crate) fn get_details(&self, name: &str) -> ServiceResult<RegistryDto> {
        STATE.with(|s| {
            let store = s.registry_store.borrow();
            let registries = store.get_registries();
            let registry = get_registry(registries, name)?;
            Ok(RegistryDto::from(registry))
        })
    }

    pub fn reclaim_name(
        &mut self,
        name: &str,
        caller: &Principal,
        new_owner: &Principal,
        resolver: &Principal,
    ) -> ServiceResult<bool> {
        must_be_named_canister(*caller, CanisterNames::Registrar)?;
        STATE.with(|s| {
            let mut store = s.registry_store.borrow_mut();
            let registries = store.get_registries_mut();

            let registry = registries.get_mut(name);
            if let Some(registry) = registry {
                registry.set_owner(new_owner.to_owned());
                registry.set_ttl(DEFAULT_TTL);
                registry.set_resolver(resolver.to_owned());
                registry.set_operators(HashSet::new());
            } else {
                let registry = Registry::new(
                    name.to_string(),
                    new_owner.to_owned(),
                    DEFAULT_TTL,
                    *resolver,
                );
                registries.insert(name.to_string(), registry);
            }
        });
        Ok(true)
    }

    async fn reset_name(
        &mut self,
        name: &str,
        caller: &Principal,
        resolver: Principal,
    ) -> ServiceResult<bool> {
        must_not_anonymous(caller)?;
        must_be_named_canister(*caller, CanisterNames::Registrar)?;
        // prevent remove top level name
        assert_ne!(name, NAMING_TOP_LABEL);
        let (sub_names, registry) = STATE.with(|s| {
            let store = s.registry_store.borrow();
            let registry = store.get_registry(name);
            if registry.is_none() {
                return Err(NamingError::RegistryNotFoundError {
                    name: name.to_string(),
                });
            }
            let registry = registry.unwrap().clone();
            let sub_names = store.get_sub_names(name);
            debug!("reset_name: sub_names: {:?}", sub_names);

            Ok((sub_names, registry))
        })?;

        let mut removing_names = sub_names;
        removing_names.push(name.to_string());

        // remove resolvers for current and sub names
        self.resolver_api
            .remove_resolvers(removing_names.clone())
            .await?;
        debug!(
            "reset_name: removed resolvers for sub_names: {:?}",
            &removing_names
        );

        STATE.with(|s| {
            let mut store = s.registry_store.borrow_mut();
            // remove registries
            store.remove_names(&removing_names);
            debug!(
                "reset_name: removed registries for sub_names: {:?}",
                &removing_names
            );

            // insert current registry
            store.add_registry(Registry::new(
                name.to_string(),
                registry.get_owner().to_owned(),
                DEFAULT_TTL,
                resolver,
            ));
            Ok(true)
        })
    }

    pub async fn transfer(
        &mut self,
        name: &str,
        caller: &Principal,
        new_owner: &Principal,
        resolver: Principal,
    ) -> ServiceResult<bool> {
        must_be_named_canister(*caller, CanisterNames::Registrar)?;
        must_not_anonymous(new_owner)?;
        must_not_anonymous(caller)?;
        self.reset_name(name, caller, resolver).await?;

        STATE.with(|s| {
            let mut store = s.registry_store.borrow_mut();
            store.update_owner(name, new_owner.to_owned());
            info!(
                "transfer: updated owner for name: {} to: {}",
                name, new_owner
            );
            Ok(true)
        })
    }
}

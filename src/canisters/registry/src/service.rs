use std::collections::HashMap;
use std::sync::Arc;

use candid::Principal;
use log::{debug, info};

use common::canister_api::ic_impl::ResolverApi;
use common::canister_api::IResolverApi;
use common::constants::{DEFAULT_TTL, TOP_LABEL};
use common::dto::{GetPageInput, GetPageOutput, IRegistryUsers, RegistryDto, RegistryUsers};
use common::errors::{ICNSError, ICNSResult};

use crate::models::*;
use crate::state::REGISTRIES;

#[cfg(test)]
mod tests;

pub struct RegistriesService {
    pub resolver_api: Arc<dyn IResolverApi>,
}

pub fn get_registry<'a>(
    registries: &'a HashMap<String, Registry>,
    name: &str,
) -> ICNSResult<&'a Registry> {
    let registry = registries.get(name);
    if let Some(registry) = registry {
        Ok(registry)
    } else {
        Err(ICNSError::RegistryNotFoundError {
            name: name.to_string(),
        })
    }
}

pub fn get_registry_mut<'a>(
    registries: &'a mut HashMap<String, Registry>,
    name: &str,
) -> ICNSResult<&'a mut Registry> {
    let registry = registries.get_mut(name);
    if let Some(registry) = registry {
        Ok(registry)
    } else {
        Err(ICNSError::RegistryNotFoundError {
            name: name.to_string(),
        })
    }
}

impl RegistriesService {
    pub fn new() -> Self {
        Self {
            resolver_api: Arc::new(ResolverApi::new()),
        }
    }

    pub fn set_top_icp_name(&mut self, registrar: Principal) -> ICNSResult<bool> {
        REGISTRIES.with(|registries| {
            let registries = registries.borrow();
            if registries.contains_key(TOP_LABEL) {
                Err(ICNSError::TopNameAlreadyExists)
            } else {
                Ok(true)
            }
        })?;

        self.set_top_name(Registry::new(
            TOP_LABEL.to_string(),
            registrar,
            DEFAULT_TTL,
            Principal::anonymous(),
        ))
    }

    fn set_top_name(&mut self, registry: Registry) -> ICNSResult<bool> {
        REGISTRIES.with(|registries| {
            let mut registries = registries.borrow_mut();
            if registries.len() > 0 {
                return Err(ICNSError::TopNameAlreadyExists);
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
    ) -> ICNSResult<RegistryDto> {
        debug!("set_subdomain_owner: label: {}, parent_name: {}, owner: {}, sub_owner: {}, ttl: {}, resolver: {}", label, parent_name, owner, sub_owner, ttl, resolver);

        REGISTRIES.with(|registries| {
            let registries = registries.borrow();
            let registry = get_registry(&registries, &parent_name)?;
            if !registry.is_owner(&owner) {
                Err(ICNSError::PermissionDenied)
            } else {
                Ok(true)
            }
        })?;

        let subdomain_name = format!("{}.{}", label, parent_name);
        // find old registry
        let updated_registry = REGISTRIES.with(|registries| {
            let mut registries = registries.borrow_mut();
            let old_registry = registries.get_mut(&subdomain_name);
            if old_registry.is_some() {
                info!("old_registry: {:?}", old_registry);
                // update owner of old registry
                let old_registry = old_registry.unwrap();
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

    pub fn check_exist(&self, name: String) -> bool {
        REGISTRIES.with(|registries| {
            let registries = registries.borrow();
            registries.contains_key(&name)
        })
    }

    pub fn get_resolver(&self, name: &str) -> ICNSResult<Principal> {
        REGISTRIES.with(|registries| {
            let registries = registries.borrow();
            let registry = get_registry(&registries, name)?;
            Ok(registry.get_resolver())
        })
    }
    pub fn set_record(
        &mut self,
        caller: &Principal,
        name: &str,
        ttl: u64,
        resolver: &Principal,
    ) -> ICNSResult<bool> {
        REGISTRIES.with(|registries| {
            let mut registries = registries.borrow_mut();
            let registry = get_registry_mut(&mut registries, &name)?;
            if !registry.can_operate(&caller) {
                return Err(ICNSError::PermissionDenied);
            }
            registry.set_ttl(ttl);
            registry.set_resolver(resolver.clone());
            Ok(true)
        })
    }

    pub(crate) fn get_controlled_names(
        &self,
        owner: Principal,
        page: GetPageInput,
    ) -> ICNSResult<GetPageOutput<String>> {
        REGISTRIES.with(|registries| {
            let registries = registries.borrow();
            let resolver_names = registries
                .iter()
                .filter_map(|(name, registry)| {
                    if registry.is_owner(&owner) {
                        Some(name.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>();

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
    ) -> ICNSResult<bool> {
        REGISTRIES.with(|registries| {
            let mut registries = registries.borrow_mut();
            let registry = get_registry_mut(&mut registries, &name)?;
            if !registry.is_owner(&caller) {
                return Err(ICNSError::PermissionDenied);
            }
            if caller == operator {
                return Err(ICNSError::OperatorShouldNotBeTheSameToOwner);
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
    ) -> ICNSResult<bool> {
        REGISTRIES.with(|registries| {
            let mut registries = registries.borrow_mut();
            let registry = get_registry_mut(&mut registries, &name)?;
            if !registry.is_owner(&caller) {
                return Err(ICNSError::PermissionDenied);
            }
            registry.remove_operator(operator);
            Ok(true)
        })
    }

    pub(crate) fn get_users(&self, name: &String) -> ICNSResult<RegistryUsers> {
        REGISTRIES.with(|registries| {
            let registries = registries.borrow();
            let registry = get_registry(&registries, name)?;
            Ok(registry.get_users())
        })
    }

    pub(crate) fn get_owner(&self, name: &String) -> ICNSResult<Principal> {
        REGISTRIES.with(|registries| {
            let registries = registries.borrow();
            let registry = get_registry(&registries, name)?;
            Ok(registry.get_owner().to_owned())
        })
    }

    pub(crate) fn get_ttl(&self, name: &String) -> ICNSResult<u64> {
        REGISTRIES.with(|registries| {
            let registries = registries.borrow();
            let registry = get_registry(&registries, name)?;
            Ok(registry.get_ttl())
        })
    }

    pub(crate) fn get_details(&self, name: &String) -> ICNSResult<RegistryDto> {
        REGISTRIES.with(|registries| {
            let registries = registries.borrow();
            let registry = get_registry(&registries, name)?;
            Ok(RegistryDto::from(registry))
        })
    }
}

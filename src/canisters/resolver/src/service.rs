use std::collections::HashMap;
use std::sync::Arc;

use log::info;

use common::canister_api::ic_impl::RegistryApi;
use common::canister_api::IRegistryApi;
use common::constants::{ALL_RESOLVER_KEYS, RESOLVER_VALUE_MAX_LENGTH};
use common::dto::IRegistryUsers;
use common::errors::*;
use common::ic_api::wrapper::ICStaticApi;
use common::ic_api::IRequestContext;

use crate::models::*;
use crate::state::RESOLVERS;

#[cfg(test)]
mod tests;

pub struct ResolverService {
    pub registry_api: Arc<dyn IRegistryApi>,
    pub request_context: Arc<dyn IRequestContext>,
}

fn get_resolver<'a>(
    resolvers: &'a HashMap<String, Resolver>,
    name: &'a str,
) -> ICNSResult<&'a Resolver> {
    match resolvers.get(name) {
        Some(resolver) => Ok(resolver),
        None => Err(ICNSError::ResolverNotFoundError {
            name: name.to_string(),
        }),
    }
}

fn get_resolver_mut<'a>(
    resolvers: &'a mut HashMap<String, Resolver>,
    name: &'a str,
) -> ICNSResult<&'a mut Resolver> {
    match resolvers.get_mut(name) {
        Some(resolver) => Ok(resolver),
        None => Err(ICNSError::ResolverNotFoundError {
            name: name.to_string(),
        }),
    }
}

impl ResolverService {
    pub fn new() -> Self {
        Self {
            registry_api: Arc::new(RegistryApi::new()),
            request_context: Arc::new(ICStaticApi::new()),
        }
    }

    pub(crate) async fn set_record_value(
        &mut self,
        name: &str,
        patch_value: &HashMap<String, String>,
    ) -> ICNSResult<bool> {
        // check resolver found
        RESOLVERS.with(|resolvers| {
            let resolvers = resolvers.borrow();
            let resolver = get_resolver(&resolvers, name);
            match resolver {
                Ok(_) => Ok(true),
                Err(e) => Err(e),
            }
        })?;

        // validate key and values
        for (key, value) in patch_value {
            if !ALL_RESOLVER_KEYS.contains(&key.as_str()) {
                return Err(ICNSError::InvalidResolverKey { key: key.clone() });
            }
            if !value.is_empty() {
                let max_length = RESOLVER_VALUE_MAX_LENGTH;
                if value.len() > max_length {
                    return Err(ICNSError::ValueMaxLengthError { max: max_length });
                }
            }
        }

        // check permission
        let caller = self.request_context.get_caller();
        let users = self.registry_api.get_users(&name).await?;
        if !users.can_operate(&caller) {
            return Err(ICNSError::PermissionDenied);
        }

        // set record value
        RESOLVERS.with(|resolvers| {
            let mut resolvers = resolvers.borrow_mut();
            let resolver = get_resolver_mut(&mut resolvers, name)?;
            for (key, value) in patch_value.iter() {
                if value.is_empty() {
                    info!("Removing resolver record {}:{}", name, key);
                    resolver.remove_record_value(key.clone());
                } else {
                    info!("Setting resolver record {}:{}", name, key);
                    resolver.set_record_value(key.clone(), value.clone());
                }
            }
            Ok(true)
        })
    }

    pub fn ensure_resolver_created(&mut self, name: &str) -> ICNSResult<bool> {
        RESOLVERS.with(|resolvers| {
            let mut resolvers = resolvers.borrow_mut();
            let name = name.to_string();
            if !resolvers.contains_key(&name) {
                resolvers.insert(name.clone(), Resolver::new(name.clone()));
                info!("Created resolver {}", name);
            } else {
                info!("Resolver {} already exists", name);
            }
            Ok(true)
        })
    }
    pub(crate) fn get_record_value(&self, name: &str) -> ICNSResult<HashMap<String, String>> {
        RESOLVERS.with(|resolvers| {
            let resolvers = resolvers.borrow();
            let resolver = get_resolver(&resolvers, &name)?;
            Ok(resolver.get_record_value().clone())
        })
    }
}

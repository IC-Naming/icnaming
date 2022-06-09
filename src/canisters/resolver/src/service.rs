use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api;
use log::info;

use common::canister_api::ic_impl::RegistryApi;
use common::canister_api::IRegistryApi;
use common::constants::{ResolverKey, RESOLVER_VALUE_MAX_LENGTH};
use common::dto::IRegistryUsers;
use common::errors::*;
use common::ic_api::wrapper::ICStaticApi;
use common::ic_api::IRequestContext;
use common::metrics_encoder::MetricsEncoder;
use common::named_canister_ids::CANISTER_NAME_REGISTRY;
use common::permissions::must_be_named_canister;

use crate::coinaddress::{validate_btc_address, validate_ltc_address};
use crate::resolver_store::*;
use crate::state::STATE;

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
        mut patch_value: HashMap<String, String>,
    ) -> ICNSResult<bool> {
        let keys = patch_value.keys().cloned().collect::<Vec<_>>();
        // validate and normalize key and value
        for key in keys {
            let resolver_key = ResolverKey::from_str(&key)?;
            let value = patch_value.get_mut(&key).unwrap();
            if !value.is_empty() {
                let max_length = RESOLVER_VALUE_MAX_LENGTH;
                if let Some(normalized_value) = normalize_value(&resolver_key, value) {
                    // update value
                    *value = normalized_value;
                }
                if value.len() > max_length {
                    return Err(ICNSError::ValueMaxLengthError { max: max_length });
                }
                validate_value(&resolver_key, value)?;
            }
        }

        // check permission
        let caller = self.request_context.get_caller();
        let users = self.registry_api.get_users(&name).await?;
        if !users.can_operate(&caller) {
            return Err(ICNSError::PermissionDenied);
        }

        // set record value
        STATE.with(|s| {
            let mut store = s.resolver_store.borrow_mut();
            store.ensure_created(name);

            let mut resolvers = store.get_resolvers_mut();
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
        STATE.with(|s| {
            let mut store = s.resolver_store.borrow_mut();
            let resolvers = store.get_resolvers_mut();
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
        STATE.with(|s| {
            let store = s.resolver_store.borrow();
            let resolvers = store.get_resolvers();
            if resolvers.get(name).is_none() {
                Ok(HashMap::new())
            } else {
                let resolver = get_resolver(&resolvers, &name)?;
                Ok(resolver.get_record_value().clone())
            }
        })
    }

    pub(crate) fn remove_resolvers(
        &self,
        caller: &Principal,
        names: Vec<String>,
    ) -> ICNSResult<bool> {
        must_be_named_canister(caller, CANISTER_NAME_REGISTRY)?;
        STATE.with(|s| {
            let mut store = s.resolver_store.borrow_mut();
            store.clean_up_names(&names);
            info!("Removing resolvers {}", &names.join(", "));
            Ok(true)
        })
    }

    pub fn get_stats(&self) -> Stats {
        let mut stats = Stats::default();
        stats.cycles_balance = api::canister_balance();
        STATE.with(|s| {
            let store = s.resolver_store.borrow();
            let resolvers = store.get_resolvers();
            stats.resolver_count = resolvers.len() as u64;
        });

        stats
    }
}

fn normalize_value(key: &ResolverKey, value: &str) -> Option<String> {
    match key {
        ResolverKey::Eth => Some(value.to_lowercase()),
        _ => None,
    }
}

fn validate_value(key: &ResolverKey, value: &str) -> ICNSResult<()> {
    match key {
        ResolverKey::Eth => {
            // validate value should be a valid eth address
            if !is_valid_eth_address(value) {
                return Err(ICNSError::InvalidResolverValueFormat {
                    value: value.to_string(),
                    format: "0x0000000000000000000000000000000000000000".to_string(),
                });
            }
        }
        ResolverKey::Btc => {
            // validate value should be a valid btc address
            if !is_valid_btc_address(value) {
                return Err(ICNSError::InvalidResolverValueFormat {
                    value: value.to_string(),
                    format: "BTC".to_string(),
                });
            }
        }
        ResolverKey::Icp => {
            // validate value should be a valid icp address
            if !is_valid_icp_address(value) {
                return Err(ICNSError::InvalidResolverValueFormat {
                    value: value.to_string(),
                    format: "principal or account id (64 digital hex string)".to_string(),
                });
            }
        }
        ResolverKey::Ltc => {
            // validate value should be a valid ltc address
            if !is_valid_ltc_address(value) {
                return Err(ICNSError::InvalidResolverValueFormat {
                    value: value.to_string(),
                    format: "LTC".to_string(),
                });
            }
        }
        ResolverKey::IcpCanister => {
            // do nothing validate since, it would be able to set custom domain for canister
        }
        ResolverKey::IcpPrincipal => {
            if !is_valid_icp_principal(value) {
                return Err(ICNSError::InvalidResolverValueFormat {
                    value: value.to_string(),
                    format: "it is no a valid principal text".to_string(),
                });
            }
        }
        ResolverKey::IcpAccountId => {
            if !is_valid_icp_account_id(value) {
                return Err(ICNSError::InvalidResolverValueFormat {
                    value: value.to_string(),
                    format: "it is no a valid account id text (64 chars hex string)".to_string(),
                });
            }
        }
        ResolverKey::Email => {
            // do nothing
        }
        ResolverKey::Url => {
            // do nothing
        }
        ResolverKey::Avatar => {
            // do nothing
        }
        ResolverKey::Description => {
            // do nothing
        }
        ResolverKey::Notice => {
            // do nothing
        }
        ResolverKey::Keywords => {
            // do nothing
        }
        ResolverKey::Twitter => {
            // do nothing
        }
        ResolverKey::Github => {
            // do nothing
        }
    }
    Ok(())
}

// impl is_valid_eth_address
fn is_valid_eth_address(address: &str) -> bool {
    if address.len() != 42 {
        return false;
    }
    let mut chars = address.chars();
    if chars.next() != Some('0') || chars.next() != Some('x') {
        return false;
    }
    for c in chars {
        if !c.is_ascii_hexdigit() {
            return false;
        }
    }
    true
}

// impl is_valid_btc_address BASE58
fn is_valid_btc_address(address: &str) -> bool {
    validate_btc_address(address).is_ok()
}

// impl is_valid_icp_address
fn is_valid_icp_address(address: &str) -> bool {
    if is_valid_icp_principal(address) {
        return true;
    }
    // ok if it's a valid 64 hex digit string
    if is_valid_icp_account_id(address) {
        return true;
    }
    false
}

fn is_valid_icp_principal(address: &str) -> bool {
    Principal::from_str(address).is_ok()
}

fn is_valid_icp_account_id(address: &str) -> bool {
    if address.len() == 64 && address.chars().all(|c| c.is_ascii_hexdigit()) {
        return true;
    }
    false
}

// impl is_valid_ltc_address
fn is_valid_ltc_address(address: &str) -> bool {
    validate_ltc_address(address).is_ok()
}

pub fn encode_metrics(w: &mut MetricsEncoder<Vec<u8>>) -> std::io::Result<()> {
    let service = ResolverService::new();
    let stats = service.get_stats();
    w.encode_gauge(
        "icnaming_resolver_cycles_balance",
        stats.cycles_balance as f64,
        "Balance in cycles",
    )?;
    w.encode_gauge(
        "icnaming_resolver_resolver_count",
        stats.resolver_count as f64,
        "Number of resolvers",
    )?;

    Ok(())
}

#[derive(CandidType, Deserialize, Default)]
pub struct Stats {
    cycles_balance: u64,
    resolver_count: u64,
}

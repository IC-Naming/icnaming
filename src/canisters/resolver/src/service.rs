use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api;
use log::{debug, info};

use common::canister_api::ic_impl::RegistryApi;
use common::canister_api::IRegistryApi;
use common::constants::{
    ResolverKey, RESOLVER_ITEM_MAX_COUNT, RESOLVER_KEY_MAX_LENGTH, RESOLVER_VALUE_MAX_LENGTH,
};
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
        let resolver = STATE.with(|s| {
            let resolver_store = s.resolver_store.borrow();
            let resolvers = resolver_store.get_resolvers();
            if let Ok(resolver) = get_resolver(resolvers, name) {
                resolver.clone()
            } else {
                Resolver::new(name.to_string())
            }
        });

        let caller = self.request_context.get_caller();
        let mut context =
            SetRecordValueValidator::new(caller, name.to_string(), patch_value, resolver);
        let input = context.validate().await?;

        input.update_state()?;
        Ok(true)
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

pub struct SetRecordValueValidator {
    caller: Principal,
    name: String,
    patch_value: HashMap<String, String>,
    resolver: Resolver,
    pub registry_api: Arc<dyn IRegistryApi>,
}

impl SetRecordValueValidator {
    pub fn new(
        caller: Principal,
        name: String,
        patch_value: HashMap<String, String>,
        resolver: Resolver,
    ) -> Self {
        Self {
            caller,
            name,
            patch_value,
            registry_api: Arc::new(RegistryApi::new()),
            resolver,
        }
    }

    fn validate_key_value(&self, key: &str, value: &str) -> ICNSResult<String> {
        if !value.is_empty() {
            {
                let max_length = RESOLVER_VALUE_MAX_LENGTH;
                if value.len() > max_length {
                    return Err(ICNSError::ValueMaxLengthError { max: max_length });
                }
            }

            {
                let max_length = RESOLVER_KEY_MAX_LENGTH;
                if key.len() > max_length {
                    return Err(ICNSError::KeyMaxLengthError { max: max_length });
                }
            }

            if let Some(resolver_key) = ResolverKey::parse(key) {
                let normalized_value = normalize_value(&resolver_key, value);
                validate_well_known_value(&resolver_key, &normalized_value)?;
                Ok(normalized_value)
            } else {
                debug!("Not well-Unknown resolver key {}", key);
                Ok(value.to_string())
            }
        } else {
            Ok(value.to_string())
        }
    }

    pub async fn validate(&self) -> ICNSResult<SetRecordValueInput> {
        let mut patch_values = vec![];
        // validate and normalize key and value
        for (key, value) in self.patch_value.iter() {
            let valid_value = self.validate_key_value(key, value)?;
            patch_values.push((key.clone(), valid_value));
        }

        // validate max item count
        let mut count_new = self.resolver.string_value_map().len();
        for (key, _) in patch_values.iter() {
            if !self.resolver.contains_key(key) {
                count_new += 1;
            }
        }
        if count_new > RESOLVER_ITEM_MAX_COUNT {
            return Err(ICNSError::TooManyResolverKeys {
                max: RESOLVER_ITEM_MAX_COUNT as u32,
            });
        }

        // check permission
        let users = self.registry_api.get_users(&self.name).await?;
        if !users.can_operate(&self.caller) {
            debug!("Permission denied for {}", self.caller);
            return Err(ICNSError::PermissionDenied);
        }

        Ok(SetRecordValueInput {
            name: self.name.clone(),
            update_records_input: patch_values
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        if v.is_empty() {
                            UpdateRecordInput::Remove
                        } else {
                            UpdateRecordInput::Set(v.clone())
                        },
                    )
                })
                .collect(),
        })
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum UpdateRecordInput {
    Set(String),
    Remove,
}

#[derive(Debug)]
pub struct SetRecordValueInput {
    pub name: String,
    pub update_records_input: HashMap<String, UpdateRecordInput>,
}

impl SetRecordValueInput {
    pub fn update_state(&self) -> ICNSResult<()> {
        STATE.with(|s| {
            // set record value
            {
                let mut store = s.resolver_store.borrow_mut();
                store.ensure_created(&self.name);
                let mut resolvers = store.get_resolvers_mut();
                let resolver = get_resolver_mut(&mut resolvers, &self.name)?;
                for (key, value) in self.update_records_input.iter() {
                    match value {
                        UpdateRecordInput::Remove => {
                            info!("Removing resolver record {}:{}", &self.name, key);
                            resolver.remove_record_value(key.clone());
                        }
                        UpdateRecordInput::Set(value) => {
                            info!("Setting resolver record {}:{}", &self.name, key);
                            resolver.set_record_value(key.clone(), value.clone());
                        }
                    }
                }
            }
            Ok(())
        })
    }
}

fn normalize_value(key: &ResolverKey, value: &str) -> String {
    match key {
        ResolverKey::Eth => value.to_lowercase(),
        _ => value.to_string(),
    }
}

fn validate_well_known_value(key: &ResolverKey, value: &str) -> ICNSResult<()> {
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
                    format: "principal or account id (64 chars hex string)".to_string(),
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

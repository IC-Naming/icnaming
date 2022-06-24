use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;

use candid::{Principal, Service};

use common::{AuthPrincipal, CallContext};
use log::{debug, info};

use common::canister_api::ic_impl::RegistryApi;
use common::canister_api::IRegistryApi;
use common::constants::{
    ResolverKey, RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL, RESOLVER_VALUE_MAX_LENGTH,
};
use common::dto::IRegistryUsers;
use common::errors::*;
use common::named_canister_ids::CanisterNames;
use common::permissions::must_not_anonymous;

use crate::coinaddress::{validate_btc_address, validate_ltc_address};
use crate::resolver_store::*;
use crate::state::STATE;

#[cfg(test)]
mod tests;

#[derive(Default)]
pub struct ResolverService {}

fn get_resolver<'a>(
    resolvers: &'a HashMap<String, Resolver>,
    name: &'a str,
) -> ServiceResult<&'a Resolver> {
    match resolvers.get(name) {
        Some(resolver) => Ok(resolver),
        None => Err(NamingError::ResolverNotFoundError {
            name: name.to_string(),
        }),
    }
}

fn get_resolver_mut<'a>(
    resolvers: &'a mut HashMap<String, Resolver>,
    name: &'a str,
) -> ServiceResult<&'a mut Resolver> {
    match resolvers.get_mut(name) {
        Some(resolver) => Ok(resolver),
        None => Err(NamingError::ResolverNotFoundError {
            name: name.to_string(),
        }),
    }
}

impl ResolverService {
    pub async fn set_record_value(
        &mut self,
        call_context: CallContext,
        name: &str,
        patch_value: HashMap<String, String>,
    ) -> ServiceResult<bool> {
        let caller = call_context.must_not_anonymous()?;

        let mut context = SetRecordValueValidator::new(caller, name.to_string(), patch_value);
        let input = context.validate().await?;

        input.update_state()?;
        Ok(true)
    }

    pub fn ensure_resolver_created(&mut self, name: &str) -> ServiceResult<bool> {
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
    pub fn get_record_value(&self, name: &str) -> ServiceResult<HashMap<String, String>> {
        STATE.with(|s| {
            let store = s.resolver_store.borrow();
            let resolvers = store.get_resolvers();
            if resolvers.get(name).is_none() {
                Ok(HashMap::new())
            } else {
                let resolver = get_resolver(&resolvers, &name)?;
                let mut values = resolver.get_record_value().clone();

                let store = s.reverse_resolver_store.borrow();
                if let Some(principal) = store.get_primary_name_reverse(&name.to_string()) {
                    values.insert(
                        RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL.to_string(),
                        principal.to_string(),
                    );
                }
                Ok(values)
            }
        })
    }

    pub fn remove_resolvers(&self, caller: CallContext, names: Vec<String>) -> ServiceResult<bool> {
        caller.must_be_named_canister(CanisterNames::Registry)?;
        STATE.with(|s| {
            let mut resolvers = vec![];
            {
                let mut store = s.resolver_store.borrow_mut();
                for name in names.iter() {
                    if let Some(resolver) = store.remove_resolver(name) {
                        resolvers.push(resolver);
                    }
                }
                info!("Removing resolvers {}", &names.join(", "));
            }

            // remove primary names
            {
                let mut store = s.reverse_resolver_store.borrow_mut();
                for resolver in resolvers.iter() {
                    let values = resolver.get_record_value();
                    if let Some(principal) =
                        values.get(RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL)
                    {
                        debug!("Removing reverse resolution principal {}", principal);
                        store.remove_primary_name(Principal::from_text(principal).unwrap());
                    }
                }
            }
            Ok(true)
        })
    }

    pub fn reverse_resolve_principal(&self, principal: Principal) -> ServiceResult<Option<String>> {
        let auth_principal = must_not_anonymous(&principal)?;

        STATE.with(|s| {
            let reverse_resolver_store = s.reverse_resolver_store.borrow();
            let value = reverse_resolver_store.get_primary_name(&auth_principal.0);
            value
                .map(|value| Ok(Some(value.clone())))
                .unwrap_or(Ok(None))
        })
    }
}

pub struct SetRecordValueValidator {
    caller: AuthPrincipal,
    name: String,
    patch_value: HashMap<String, String>,
    pub registry_api: Arc<dyn IRegistryApi>,
}

impl SetRecordValueValidator {
    pub fn new(caller: AuthPrincipal, name: String, patch_value: HashMap<String, String>) -> Self {
        Self {
            caller,
            name,
            patch_value,
            registry_api: Arc::new(RegistryApi::default()),
        }
    }

    fn validate_key_value(&self, resolver_key: &ResolverKey, value: &str) -> ServiceResult<String> {
        if !value.is_empty() {
            let max_length = RESOLVER_VALUE_MAX_LENGTH;
            let normalized_value = normalize_value(&resolver_key, value);
            // update value
            if normalized_value.len() > max_length {
                return Err(NamingError::ValueMaxLengthError { max: max_length });
            }
            validate_value(&resolver_key, &normalized_value)?;
            Ok(normalized_value)
        } else {
            Ok(value.to_string())
        }
    }

    pub async fn validate(&self) -> ServiceResult<SetRecordValueInput> {
        let mut patch_values = vec![];
        // validate and normalize key and value
        let mut update_primary_name_input_value = self
            .patch_value
            .get(RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL)
            .cloned();

        for (key, value) in self.patch_value.iter() {
            let resolver_key = ResolverKey::from_str(key)?;
            let valid_value = self.validate_key_value(&resolver_key, value)?;
            if key != RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL {
                patch_values.push((key.clone(), valid_value));
            }
        }

        // check permission
        let users = self.registry_api.get_users(&self.name).await?;
        if !users.can_operate(&self.caller.0) {
            debug!("Permission denied for {}", self.caller.0);
            return Err(NamingError::PermissionDenied);
        }

        // check ResolverKey::SettingReverseResolutionPrincipal
        if update_primary_name_input_value.is_some() {
            if &self.caller.0 != users.get_owner() {
                debug!(
                    "SettingReverseResolutionPrincipal is not allowed since caller is not owner"
                );
                return Err(NamingError::PermissionDenied);
            }
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
            update_primary_name_input: if let Some(principalString) =
                update_primary_name_input_value
            {
                if principalString.is_empty() {
                    UpdatePrimaryNameInput::Remove(self.caller.0.clone())
                } else {
                    UpdatePrimaryNameInput::Set(self.caller.0.clone())
                }
            } else {
                UpdatePrimaryNameInput::DoNothing
            },
        })
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum UpdatePrimaryNameInput {
    DoNothing,
    Set(Principal),
    Remove(Principal),
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
    pub update_primary_name_input: UpdatePrimaryNameInput,
}

impl SetRecordValueInput {
    pub fn update_state(&self) -> ServiceResult<()> {
        STATE.with(|s| {
            // set primary name
            {
                let mut store = s.reverse_resolver_store.borrow_mut();
                match self.update_primary_name_input {
                    UpdatePrimaryNameInput::DoNothing => {
                        info!("Doing nothing for reverse resolution principal");
                    }
                    UpdatePrimaryNameInput::Set(value) => {
                        info!(
                            "Setting reverse resolution principal {} {}",
                            self.name, value
                        );
                        store.set_primary_name(value, self.name.clone());
                    }
                    UpdatePrimaryNameInput::Remove(value) => {
                        info!("Removing reverse resolution principal {}", value);
                        store.remove_primary_name(value);
                    }
                }
            }
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

fn validate_value(key: &ResolverKey, value: &str) -> ServiceResult<()> {
    match key {
        ResolverKey::Eth => {
            // validate value should be a valid eth address
            if !is_valid_eth_address(value) {
                return Err(NamingError::InvalidResolverValueFormat {
                    value: value.to_string(),
                    format: "0x0000000000000000000000000000000000000000".to_string(),
                });
            }
        }
        ResolverKey::Btc => {
            // validate value should be a valid btc address
            if !is_valid_btc_address(value) {
                return Err(NamingError::InvalidResolverValueFormat {
                    value: value.to_string(),
                    format: "BTC".to_string(),
                });
            }
        }
        ResolverKey::Icp => {
            // validate value should be a valid icp address
            if !is_valid_icp_address(value) {
                return Err(NamingError::InvalidResolverValueFormat {
                    value: value.to_string(),
                    format: "principal or account id (64 chars hex string)".to_string(),
                });
            }
        }
        ResolverKey::Ltc => {
            // validate value should be a valid ltc address
            if !is_valid_ltc_address(value) {
                return Err(NamingError::InvalidResolverValueFormat {
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
                return Err(NamingError::InvalidResolverValueFormat {
                    value: value.to_string(),
                    format: "it is no a valid principal text".to_string(),
                });
            }
        }
        ResolverKey::IcpAccountId => {
            if !is_valid_icp_account_id(value) {
                return Err(NamingError::InvalidResolverValueFormat {
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
        ResolverKey::SettingReverseResolutionPrincipal => {
            if !is_valid_icp_principal(value) {
                return Err(NamingError::InvalidResolverValueFormat {
                    value: value.to_string(),
                    format: "it is no a valid principal text".to_string(),
                });
            }
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

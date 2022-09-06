use std::collections::HashMap;
use std::hash::Hash;
use std::str::FromStr;
use std::sync::Arc;
use std::vec::Vec;

use candid::Principal;
use ic_cdk::{call, caller};

use common::{AuthPrincipal, CallContext};
use log::{debug, info};

use common::canister_api::ic_impl::RegistryApi;
use common::canister_api::IRegistryApi;
use common::constants::{
    WellKnownResolverKey, RESOLVER_ITEM_MAX_COUNT, RESOLVER_KEY_MAX_LENGTH,
    RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL, RESOLVER_VALUE_MAX_LENGTH,
};
use common::dto::IRegistryUsers;
use common::errors::*;

use common::named_canister_ids::{is_named_canister_id, CanisterNames};
use common::permissions::{must_be_system_owner, must_not_anonymous};

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
        patch_values: HashMap<String, String>,
    ) -> ServiceResult<bool> {
        let caller = call_context.must_not_anonymous()?;

        let resolver = STATE.with(|s| {
            let resolver_store = s.resolver_store.borrow();
            let resolvers = resolver_store.get_resolvers();
            if let Ok(resolver) = get_resolver(resolvers, name) {
                resolver.clone()
            } else {
                Resolver::new(name.to_string())
            }
        });
        let patch_values: PatchValuesInput = patch_values.into();

        let patch_value_validator = PatchValuesValidator::new(name.to_string(), patch_values);
        let owner_validator = patch_value_validator.owner_validate(caller, resolver)?;

        let owner_validator = owner_validator.validate().await?;
        let input = owner_validator.generate()?;

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
                for name in names.iter() {
                    if let Some(principal) = store.remove_primary_name_by_name(name) {
                        debug!("Removing reverse resolution principal {}", principal);
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

    pub fn batch_get_reverse_resolve_principal(
        &self,
        principals: Vec<Principal>,
    ) -> ServiceResult<HashMap<Principal, Option<String>>> {
        let mut auth_principals = Vec::new();
        for principal in principals {
            auth_principals.push(must_not_anonymous(&principal)?);
        }
        STATE.with(|s| {
            let reverse_resolver_store = s.reverse_resolver_store.borrow();
            let result = auth_principals
                .iter()
                .map(|auth_principal| {
                    let value = reverse_resolver_store.get_primary_name(&auth_principal.0);
                    match value {
                        Some(value) => (auth_principal.0, Some(value.clone())),
                        None => (auth_principal.0, None),
                    }
                })
                .collect();
            Ok(result)
        })
    }

    pub fn import_record_value(
        &self,
        call_context: &CallContext,
        items: Vec<ResolverValueImportItem>,
    ) -> ServiceResult<()> {
        let _ = call_context.must_be_system_owner()?;

        let mut list = Vec::new();
        for item in items {
            let name = item.name.clone();
            let patch_values = item.into();
            let patch_values_validator: PatchValuesValidator =
                PatchValuesValidator::new(name, patch_values);
            let input_generator = patch_values_validator.resolver_value_import_validate()?;
            let input = input_generator.generate()?;
            list.push(input);
        }
        for input in list {
            input.update_state()?;
        }

        Ok(())
    }
}

pub struct PatchValuesInput(HashMap<String, PatchValueOperation>);

impl From<HashMap<String, String>> for PatchValuesInput {
    fn from(map: HashMap<String, String>) -> Self {
        let mut result = HashMap::new();
        for (key, value) in map {
            if value.is_empty() {
                result.insert(key, PatchValueOperation::Remove(value));
            } else {
                result.insert(key, PatchValueOperation::Upsert(value));
            }
        }
        PatchValuesInput(result)
    }
}

impl From<ResolverValueImportItem> for PatchValuesInput {
    fn from(item: ResolverValueImportItem) -> Self {
        let mut result = HashMap::new();
        result.insert(item.key, item.value_and_operation);
        PatchValuesInput(result)
    }
}

pub struct ResolverValueImportItem {
    pub name: String,
    pub key: String,
    pub value_and_operation: PatchValueOperation,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum PatchValueOperation {
    Upsert(String),
    InsertOrIgnore(String),
    Remove(String),
}

impl PatchValueOperation {
    pub fn get_value(&self) -> &String {
        match self {
            PatchValueOperation::Upsert(value) => value,
            PatchValueOperation::InsertOrIgnore(value) => value,
            PatchValueOperation::Remove(value) => value,
        }
    }
}

impl From<PatchValueOperation> for UpdatePrimaryNameInput {
    fn from(item: PatchValueOperation) -> Self {
        match item {
            PatchValueOperation::Upsert(value) => {
                UpdatePrimaryNameInput::Set(Principal::from_text(value).unwrap())
            }
            PatchValueOperation::InsertOrIgnore(value) => {
                UpdatePrimaryNameInput::InsertOrIgnore(Principal::from_text(value).unwrap())
            }
            PatchValueOperation::Remove(value) => {
                UpdatePrimaryNameInput::Remove(Principal::from_text(value).unwrap())
            }
        }
    }
}

pub struct PatchValuesValidator {
    pub name: String,
    pub patch_values: PatchValuesInput,
}

impl PatchValuesValidator {
    pub fn new(name: String, patch_values: PatchValuesInput) -> Self {
        Self { name, patch_values }
    }

    fn patch_values_validate(
        &self,
        resolver: Option<Resolver>,
    ) -> ServiceResult<HashMap<String, UpdateRecordInput>> {
        let mut patch_values = HashMap::new();
        // validate and normalize key and value

        for (key, value) in self.patch_values.0.iter() {
            let _ = self.validate_key_value(key, value.get_value())?;
            if key != RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL {
                match value {
                    PatchValueOperation::Upsert(value) => {
                        patch_values.insert(key.clone(), UpdateRecordInput::Set(value.clone()));
                    }
                    PatchValueOperation::InsertOrIgnore(value) => {
                        patch_values.insert(
                            key.clone(),
                            UpdateRecordInput::InsertOrIgnore(value.clone()),
                        );
                    }
                    PatchValueOperation::Remove(_) => {
                        patch_values.insert(key.clone(), UpdateRecordInput::Remove);
                    }
                }
            }
        }

        // validate max item count
        if let Some(resolver) = resolver {
            let mut count_new = resolver.string_value_map().len();
            for (key, _) in patch_values.iter() {
                if !resolver.contains_key(key) {
                    count_new += 1;
                }
            }
            if count_new > RESOLVER_ITEM_MAX_COUNT {
                return Err(NamingError::TooManyResolverKeys {
                    max: RESOLVER_ITEM_MAX_COUNT as u32,
                });
            }
        }

        Ok(patch_values)
    }
    fn validate_key_value(&self, key: &str, value: &str) -> ServiceResult<String> {
        if !value.is_empty() {
            {
                let max_length = RESOLVER_VALUE_MAX_LENGTH;
                if value.len() > max_length {
                    return Err(NamingError::ValueMaxLengthError { max: max_length });
                }
            }

            {
                let max_length = RESOLVER_KEY_MAX_LENGTH;
                if key.len() > max_length {
                    return Err(NamingError::KeyMaxLengthError { max: max_length });
                }
            }

            if let Some(resolver_key) = WellKnownResolverKey::parse(key) {
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

    fn get_update_primary_name_input(&self) -> Option<PatchValueOperation> {
        self.patch_values
            .0
            .get(RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL)
            .cloned()
    }

    fn get_remove_all(&self) -> Option<HashMap<String, UpdateRecordInput>> {
        if let Some(value) = self
            .patch_values
            .0
            .get(RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL)
        {
            match value {
                PatchValueOperation::Remove(value) => {
                    if value.is_empty() {
                        let mut patch_values = HashMap::new();
                        patch_values.insert(
                            RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL.to_string(),
                            UpdateRecordInput::Remove,
                        );
                        Some(patch_values)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn owner_validate(
        &self,
        caller: AuthPrincipal,
        resolver: Resolver,
    ) -> ServiceResult<SetRecordByOwnerValidator> {
        let patch_values = self.patch_values_validate(Some(resolver))?;
        let update_primary_name_input_value = self.get_update_primary_name_input();

        Ok(SetRecordByOwnerValidator::new(
            caller,
            self.name.clone(),
            patch_values,
            update_primary_name_input_value,
        ))
    }

    pub fn resolver_value_import_validate(&self) -> ServiceResult<SetRecordValueInputGenerator> {
        if let Some(patch_values) = self.get_remove_all() {
            return Ok(SetRecordValueInputGenerator::new(
                self.name.clone(),
                patch_values,
                None,
            ));
        } else {
            let patch_values = self.patch_values_validate(None)?;
            let update_primary_name_input_value = self.get_update_primary_name_input();

            Ok(SetRecordValueInputGenerator::new(
                self.name.clone(),
                patch_values,
                update_primary_name_input_value,
            ))
        }
    }
}

pub struct SetRecordByOwnerValidator {
    pub caller: AuthPrincipal,
    pub name: String,
    pub patch_values: HashMap<String, UpdateRecordInput>,
    pub update_primary_name_input_value: Option<PatchValueOperation>,
    pub registry_api: Arc<dyn IRegistryApi>,
}

impl SetRecordByOwnerValidator {
    pub fn new(
        caller: AuthPrincipal,
        name: String,
        patch_values: HashMap<String, UpdateRecordInput>,
        update_primary_name_input_value: Option<PatchValueOperation>,
    ) -> Self {
        Self {
            caller,
            name,
            patch_values,
            update_primary_name_input_value,
            registry_api: Arc::new(RegistryApi::default()),
        }
    }

    pub async fn validate(&self) -> ServiceResult<SetRecordValueInputGenerator> {
        let users = self.registry_api.get_users(&self.name).await?;
        let owner = users.get_owner();

        let owner = if is_named_canister_id(CanisterNames::Registrar, self.caller.0) {
            owner.clone()
        } else {
            // check permission
            if !users.can_operate(&self.caller.0) {
                debug!("Permission denied for {}", self.caller.0);
                return Err(NamingError::PermissionDenied);
            }

            // check ResolverKey::SettingReverseResolutionPrincipal
            if self.update_primary_name_input_value.is_some() {
                if &self.caller.0 != owner {
                    debug!(
                    "SettingReverseResolutionPrincipal is not allowed since caller is not owner"
                );
                    return Err(NamingError::PermissionDenied);
                }
            }
            self.caller.0.clone()
        };

        Ok(SetRecordValueInputGenerator::new(
            self.name.clone(),
            self.patch_values
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            match self.update_primary_name_input_value {
                Some(PatchValueOperation::Upsert(_)) => {
                    Some(PatchValueOperation::Upsert(owner.to_text()))
                }
                Some(PatchValueOperation::Remove(_)) => {
                    Some(PatchValueOperation::Remove(owner.to_text()))
                }
                Some(PatchValueOperation::InsertOrIgnore(_)) => {
                    Some(PatchValueOperation::Upsert(owner.to_text()))
                }
                _ => None,
            },
        ))
    }
}

pub struct SetRecordValueInputGenerator {
    pub name: String,
    pub patch_values: HashMap<String, UpdateRecordInput>,
    pub update_primary_name_input: Option<PatchValueOperation>,
}

impl SetRecordValueInputGenerator {
    pub fn new(
        name: String,
        patch_values: HashMap<String, UpdateRecordInput>,
        update_primary_name_input: Option<PatchValueOperation>,
    ) -> Self {
        Self {
            name,
            patch_values,
            update_primary_name_input,
        }
    }

    pub fn generate(&self) -> ServiceResult<SetRecordValueInput> {
        Ok(SetRecordValueInput {
            name: self.name.clone(),
            update_records_input: self.patch_values.clone(),
            update_primary_name_input: if let Some(value) = self.update_primary_name_input.clone() {
                value.into()
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
    InsertOrIgnore(Principal),
    Remove(Principal),
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum UpdateRecordInput {
    Set(String),
    InsertOrIgnore(String),
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
                    UpdatePrimaryNameInput::InsertOrIgnore(value) => {
                        info!(
                            "Insert or ignore reverse resolution principal {} {}",
                            self.name, value
                        );
                        if store.get_primary_name(&value).is_none() {
                            store.set_primary_name(value, self.name.clone());
                        }
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
                        UpdateRecordInput::InsertOrIgnore(value) => {
                            info!("Inserting resolver record {}:{}", &self.name, key);
                            if !resolver.has_record_value(key) {
                                resolver.set_record_value(key.clone(), value.clone());
                            }
                        }
                    }
                }
            }
            Ok(())
        })
    }
}

fn normalize_value(key: &WellKnownResolverKey, value: &str) -> String {
    match key {
        WellKnownResolverKey::Eth => value.to_lowercase(),
        _ => value.to_string(),
    }
}

fn validate_well_known_value(key: &WellKnownResolverKey, value: &str) -> ServiceResult<()> {
    match key {
        WellKnownResolverKey::Eth => {
            // validate value should be a valid eth address
            if !is_valid_eth_address(value) {
                return Err(NamingError::InvalidResolverValueFormat {
                    value: value.to_string(),
                    format: "0x0000000000000000000000000000000000000000".to_string(),
                });
            }
        }
        WellKnownResolverKey::Btc => {
            // validate value should be a valid btc address
            if !is_valid_btc_address(value) {
                return Err(NamingError::InvalidResolverValueFormat {
                    value: value.to_string(),
                    format: "BTC".to_string(),
                });
            }
        }
        WellKnownResolverKey::Icp => {
            // validate value should be a valid icp address
            if !is_valid_icp_address(value) {
                return Err(NamingError::InvalidResolverValueFormat {
                    value: value.to_string(),
                    format: "principal or account id (64 chars hex string)".to_string(),
                });
            }
        }
        WellKnownResolverKey::Ltc => {
            // validate value should be a valid ltc address
            if !is_valid_ltc_address(value) {
                return Err(NamingError::InvalidResolverValueFormat {
                    value: value.to_string(),
                    format: "LTC".to_string(),
                });
            }
        }
        WellKnownResolverKey::IcpCanister => {
            // do nothing validate since, it would be able to set custom domain for canister
        }
        WellKnownResolverKey::IcpPrincipal => {
            if !is_valid_icp_principal(value) {
                return Err(NamingError::InvalidResolverValueFormat {
                    value: value.to_string(),
                    format: "it is no a valid principal text".to_string(),
                });
            }
        }
        WellKnownResolverKey::IcpAccountId => {
            if !is_valid_icp_account_id(value) {
                return Err(NamingError::InvalidResolverValueFormat {
                    value: value.to_string(),
                    format: "it is no a valid account id text (64 chars hex string)".to_string(),
                });
            }
        }
        WellKnownResolverKey::SettingReverseResolutionPrincipal => {
            if !is_valid_icp_principal(value) {
                return Err(NamingError::InvalidResolverValueFormat {
                    value: value.to_string(),
                    format: "it is no a valid principal text".to_string(),
                });
            }
        }
        WellKnownResolverKey::Email => {
            // do nothing
        }
        WellKnownResolverKey::Url => {
            // do nothing
        }
        WellKnownResolverKey::Avatar => {
            // do nothing
        }
        WellKnownResolverKey::Description => {
            // do nothing
        }
        WellKnownResolverKey::Notice => {
            // do nothing
        }
        WellKnownResolverKey::Keywords => {
            // do nothing
        }
        WellKnownResolverKey::Twitter => {
            // do nothing
        }
        WellKnownResolverKey::Github => {
            // do nothing
        }
        WellKnownResolverKey::Facebook => {
            // do nothing
        }
        WellKnownResolverKey::Medium => {
            // do nothing
        }
        WellKnownResolverKey::Discord => {
            // do nothing
        }
        WellKnownResolverKey::Telegram => {
            // do nothing
        }
        WellKnownResolverKey::Instagram => {
            // do nothing
        }
        WellKnownResolverKey::Reddit => {
            // do nothing
        }
        WellKnownResolverKey::Location => {
            // do nothing
        }
        WellKnownResolverKey::DisplayName => {
            // do nothing
        }
        WellKnownResolverKey::Dscvr => {
            // do nothing
        }
        WellKnownResolverKey::Distrikt => {
            // do nothing
        }
        WellKnownResolverKey::Relation => {
            // do nothing
        }
        WellKnownResolverKey::OpenChat => {
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

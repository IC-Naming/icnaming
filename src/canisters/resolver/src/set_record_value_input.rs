use crate::coinaddress::{validate_btc_address, validate_ltc_address};
use crate::resolver_store::Resolver;
use crate::state::STATE;
use candid::{CandidType, Deserialize, Principal};
use common::canister_api::ic_impl::RegistryApi;
use common::canister_api::IRegistryApi;
use common::constants::{
    WellKnownResolverKey, RESOLVER_ITEM_MAX_COUNT, RESOLVER_KEY_MAX_LENGTH,
    RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL, RESOLVER_VALUE_MAX_LENGTH,
};
use common::dto::IRegistryUsers;
use common::errors::{NamingError, ServiceResult};
use common::named_canister_ids::{is_named_canister_id, CanisterNames};
use common::AuthPrincipal;

use log::{debug, info};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

#[cfg(test)]
mod tests;

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

#[derive(Clone, Debug)]
pub struct PatchValuesInput(pub HashMap<String, PatchValueOperation>);

impl From<HashMap<String, String>> for PatchValuesInput {
    fn from(map: HashMap<String, String>) -> Self {
        let mut result = HashMap::new();
        for (key, value) in map {
            if value.is_empty() {
                result.insert(key, PatchValueOperation::Remove);
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

#[derive(Debug, Deserialize, CandidType)]
pub struct ResolverValueImportItem {
    pub name: String,
    pub key: String,
    pub value_and_operation: PatchValueOperation,
}

#[derive(Clone, Debug)]
pub struct ResolverValueImportGroup {
    pub name: String,
    pub patch_values: Vec<PatchValuesInput>,
}

#[derive(Debug, Deserialize, CandidType, Clone, PartialEq, Eq)]
pub enum PatchValueOperation {
    Upsert(String),
    InsertOrIgnore(String),
    Remove,
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
            PatchValueOperation::Remove => UpdatePrimaryNameInput::Remove,
        }
    }
}

pub struct PatchValuesValidator {
    pub name: String,
    pub patch_values: PatchValuesInput,
    pub resolver: Resolver,
}

impl PatchValuesValidator {
    pub fn new(name: String, patch_values: PatchValuesInput, resolver: Resolver) -> Self {
        Self {
            name,
            patch_values,
            resolver,
        }
    }

    fn validate_patch_values(&self) -> ServiceResult<HashMap<String, UpdateRecordInput>> {
        let mut patch_values = HashMap::new();
        // validate and normalize key and value

        for (key, value) in self.patch_values.0.iter() {
            if key != RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL {
                match value {
                    PatchValueOperation::Upsert(value) => {
                        let value = self.validate_key_value(key, value)?;
                        patch_values.insert(key.clone(), UpdateRecordInput::Set(value.clone()));
                    }
                    PatchValueOperation::InsertOrIgnore(value) => {
                        let value = self.validate_key_value(key, value)?;
                        patch_values.insert(
                            key.clone(),
                            UpdateRecordInput::InsertOrIgnore(value.clone()),
                        );
                    }
                    PatchValueOperation::Remove => {
                        let _ = self.validate_key_value(key, &"")?;
                        patch_values.insert(key.clone(), UpdateRecordInput::Remove);
                    }
                }
            }
        }

        // validate max item count
        let mut count_new = self.resolver.string_value_map().len();
        for (key, _) in patch_values.iter() {
            if !self.resolver.contains_key(key) {
                count_new += 1;
            }
        }
        if count_new > RESOLVER_ITEM_MAX_COUNT {
            return Err(NamingError::TooManyResolverKeys {
                max: RESOLVER_ITEM_MAX_COUNT as u32,
            });
        }

        Ok(patch_values)
    }
    fn validate_key_value(&self, key: &str, value: &str) -> ServiceResult<String> {
        let max_length = RESOLVER_KEY_MAX_LENGTH;
        if key.len() > max_length {
            return Err(NamingError::KeyMaxLengthError { max: max_length });
        }

        if !value.is_empty() {
            {
                let max_length = RESOLVER_VALUE_MAX_LENGTH;
                if value.len() > max_length {
                    return Err(NamingError::ValueMaxLengthError { max: max_length });
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

    fn validate_update_primary_name_input(&self) -> ServiceResult<UpdatePrimaryNameInput> {
        return match self
            .patch_values
            .0
            .get(RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL)
        {
            Some(PatchValueOperation::Upsert(value)) => {
                validate_well_known_value(&WellKnownResolverKey::IcpPrincipal, value)?;
                Ok(UpdatePrimaryNameInput::Set(
                    Principal::from_text(value).unwrap(),
                ))
            }
            Some(PatchValueOperation::InsertOrIgnore(value)) => {
                validate_well_known_value(&WellKnownResolverKey::IcpPrincipal, value)?;
                Ok(UpdatePrimaryNameInput::InsertOrIgnore(
                    Principal::from_text(value).unwrap(),
                ))
            }
            Some(PatchValueOperation::Remove) => Ok(UpdatePrimaryNameInput::Remove),
            None => Ok(UpdatePrimaryNameInput::DoNothing),
        };
    }
    fn get_update_primary_name_input(&self) -> Option<PatchValueOperation> {
        self.patch_values
            .0
            .get(RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL)
            .cloned()
    }

    pub fn validate_and_generate_owner_validator(
        self,
        caller: AuthPrincipal,
    ) -> ServiceResult<SetRecordByOwnerValidator> {
        let patch_values = self.validate_patch_values()?;
        let update_primary_name_input_value = self.get_update_primary_name_input();

        Ok(SetRecordByOwnerValidator::new(
            caller,
            self.name.clone(),
            patch_values,
            update_primary_name_input_value,
        ))
    }
    pub fn validate_and_generate_input_generator(self) -> ServiceResult<SetRecordValueInput> {
        let patch_values = self.validate_patch_values()?;
        let update_primary_name_input_value = self.validate_update_primary_name_input()?;
        Ok(SetRecordValueInput::new(
            self.name.clone(),
            patch_values,
            update_primary_name_input_value,
        ))
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

    pub async fn validate(self) -> ServiceResult<SetRecordValueInput> {
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

        Ok(SetRecordValueInput::new(
            self.name.clone(),
            self.patch_values
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            if let Some(update_primary_name_input_value) =
                self.update_primary_name_input_value.clone()
            {
                match update_primary_name_input_value {
                    PatchValueOperation::Upsert(_value) => UpdatePrimaryNameInput::Set(owner),
                    PatchValueOperation::InsertOrIgnore(_value) => {
                        UpdatePrimaryNameInput::InsertOrIgnore(owner)
                    }
                    PatchValueOperation::Remove => UpdatePrimaryNameInput::Remove,
                }
            } else {
                UpdatePrimaryNameInput::DoNothing
            },
        ))
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum UpdatePrimaryNameInput {
    DoNothing,
    Set(Principal),
    InsertOrIgnore(Principal),
    Remove,
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
    pub fn new(
        name: String,
        update_records_input: HashMap<String, UpdateRecordInput>,
        update_primary_name_input: UpdatePrimaryNameInput,
    ) -> Self {
        Self {
            name,
            update_records_input,
            update_primary_name_input,
        }
    }

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
                            "Inserting or ignore reverse resolution principal {} {}",
                            self.name, value
                        );
                        if !store.has_primary_name_reverse(self.name.clone()) {
                            store.set_primary_name(value, self.name.clone());
                        }
                    }
                    UpdatePrimaryNameInput::Remove => {
                        info!(
                            "Removing reverse resolution principal by name {}",
                            self.name
                        );
                        store.remove_primary_name_by_name(&self.name);
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
                            info!("Inserting or ignore resolver record {}:{}", &self.name, key);
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

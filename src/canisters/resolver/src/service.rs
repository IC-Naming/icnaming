use std::collections::HashMap;

use std::vec::Vec;

use candid::{CandidType, Deserialize, Principal};
use itertools::Itertools;

use common::CallContext;
use log::{debug, error, info};

use common::constants::RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL;

use common::errors::*;

use common::named_canister_ids::CanisterNames;
use common::permissions::must_not_anonymous;

use crate::resolver_store::*;
use crate::set_record_value_input::{
    PatchValueOperation, PatchValuesInput, PatchValuesValidator, ResolverValueImportGroup,
    ResolverValueImportItem,
};
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

        let patch_value_validator =
            PatchValuesValidator::new(name.to_string(), patch_values, resolver);
        let owner_validator =
            patch_value_validator.validate_and_generate_owner_validator(caller)?;

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
        request: &ImportRecordValueRequest,
    ) -> ServiceResult<bool> {
        let _ = call_context.must_be_system_owner()?;
        let mut list = Vec::new();

        let group = request.group_up_resolver_value_import_items();

        let items: Vec<(ResolverValueImportGroup, Resolver)> = STATE.with(|s| {
            let resolvers_store = s.resolver_store.borrow();
            let resolvers = resolvers_store.get_resolvers();
            group
                .iter()
                .map(|item| match get_resolver(resolvers, &item.name) {
                    Ok(resolver) => (item.clone(), resolver.clone()),
                    Err(_) => (item.clone(), Resolver::new(item.name.clone())),
                })
                .collect::<Vec<_>>()
        });

        for item in items {
            let name = item.0.name;
            let patch_values = item.0.patch_values;
            let resolver = item.1;

            for patch_value in patch_values {
                let patch_values_validator: PatchValuesValidator =
                    PatchValuesValidator::new(name.clone(), patch_value, resolver.clone());
                let input_generator =
                    patch_values_validator.validate_and_generate_input_generator()?;
                let input = input_generator.generate()?;
                list.push(input);
            }
        }

        for input in list {
            let result = input.update_state();
            match result {
                Ok(_) => info!("Imported resolver value: {:?}", input),
                Err(err) => {
                    error!(
                        "Failed to import resolver value: {:?}, error:{:?}",
                        input, err
                    );
                    return Err(err);
                }
            }
        }
        Ok(true)
    }
}

#[derive(Debug, Deserialize, CandidType)]
pub struct ImportRecordValueRequest {
    pub items: Vec<ResolverValueImportItem>,
}

impl ImportRecordValueRequest {
    pub fn group_up_resolver_value_import_items(&self) -> Vec<ResolverValueImportGroup> {
        let mut result = Vec::new();
        self.items
            .iter()
            .group_by(|item| item.name.clone())
            .into_iter()
            .for_each(|(name, items)| {
                let map_list: Vec<HashMap<String, PatchValueOperation>> =
                    items.into_iter().fold(Vec::new(), |mut acc, item| {
                        for (i, x) in acc.iter().enumerate() {
                            if !x.contains_key(&item.key) {
                                acc.get_mut(i)
                                    .unwrap()
                                    .insert(item.key.clone(), item.value_and_operation.clone());
                                return acc;
                            }
                        }
                        let mut map = HashMap::new();
                        map.insert(item.key.clone(), item.value_and_operation.clone());
                        acc.push(map);
                        acc
                    });
                let patch_values = map_list
                    .into_iter()
                    .map(|map| PatchValuesInput(map))
                    .collect::<Vec<PatchValuesInput>>();

                result.push(ResolverValueImportGroup { name, patch_values });
            });
        result
    }
}

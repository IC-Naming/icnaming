use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use candid::{CandidType, Deserialize, Nat, Principal};

use log::{debug, error, info, trace};
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use time::{OffsetDateTime, Time};

use common::canister_api::ic_impl::{CyclesMintingApi, RegistryApi, ResolverApi};
use common::canister_api::{AccountIdentifier, ICyclesMintingApi, IRegistryApi, IResolverApi};
use common::constants::*;
use common::dto::{
    BatchAddQuotaRequest, GetPageInput, GetPageOutput, ImportQuotaRequest, ImportQuotaStatus,
};
use common::errors::{NamingError, ServiceResult};
use common::named_canister_ids::{get_named_get_canister_id, CanisterNames};
use common::named_principals::{PRINCIPAL_NAME_STATE_EXPORTER, PRINCIPAL_NAME_TIMER_TRIGGER};
use common::naming::{normalize_name, FirstLevelName, NameParseResult};
use common::nft::{CommonError, Metadata, NFTServiceResult, NFTTransferServiceResult, NonFungible};
use common::permissions::{
    must_be_in_named_canister, must_be_named_canister, must_be_system_owner,
};
use common::permissions::{must_be_named_principal, must_not_anonymous};
use common::token_identifier::{encode_token_id, get_valid_token_index, TokenIdentifier};
use common::{AuthPrincipal, CallContext, TimeInNs};

use crate::name_locker::{try_lock_name, unlock_name};
use crate::registration_store::{
    Registration, RegistrationDetails, RegistrationDto, RegistrationStore,
};
use crate::reserved_list::RESERVED_NAMES;
use crate::state::*;
use crate::token_index_store::{RegistrationName, TokenIndexStore, UnexpiredRegistrationAggDto};
use crate::token_service::TokenService;
use crate::user_quota_store::{QuotaType, TransferQuotaDetails};

#[derive(Deserialize, CandidType, Debug)]
pub struct RegisterNameWithPaymentRequest {
    pub name: String,
    pub years: u32,
    pub approve_amount: Nat,
}

pub struct RegistrarService {
    pub registry_api: Arc<dyn IRegistryApi>,
    pub cycles_minting_api: Arc<dyn ICyclesMintingApi>,
    pub token_service: TokenService,
    pub resolver_api: Arc<dyn IResolverApi>,
}

impl Debug for RegistrarService {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, stringify!(RegistrarService))
    }
}

impl Default for RegistrarService {
    fn default() -> Self {
        RegistrarService {
            registry_api: Arc::new(RegistryApi),
            cycles_minting_api: Arc::new(CyclesMintingApi),
            token_service: TokenService::default(),
            resolver_api: Arc::new(ResolverApi),
        }
    }
}

impl RegistrarService {
    pub(crate) fn get_names(
        &self,
        owner: &Principal,
        input: &GetPageInput,
    ) -> ServiceResult<GetPageOutput<RegistrationDto>> {
        input.validate()?;
        must_not_anonymous(owner)?;

        let result = STATE.with(|s| {
            let store = s.registration_store.borrow();

            let items = store
                .get_registrations()
                .values()
                .filter(|registration| registration.is_owner(owner))
                .skip(input.offset)
                .take(input.limit)
                .map(|registration| registration.into())
                .collect();
            GetPageOutput::new(items)
        });

        Ok(result)
    }

    pub(crate) fn get_names_count(&self, owner: &Principal) -> ServiceResult<u32> {
        must_not_anonymous(owner)?;

        STATE.with(|s| {
            let store = s.registration_store.borrow();
            let total = store
                .get_registrations()
                .values()
                .filter(|registration| registration.is_owner(owner))
                .count() as u32;
            Ok(total)
        })
    }

    pub(crate) fn get_details(&self, name: &str) -> ServiceResult<RegistrationDetails> {
        let name = normalize_name(name);
        STATE.with(|s| {
            let store = s.registration_store.borrow();
            let registrations = store.get_registrations();
            let registration = registrations.get(&name.0);
            if registration.is_none() {
                return Err(NamingError::RegistrationNotFound);
            }
            Ok(RegistrationDetails::from(registration.unwrap()))
        })
    }

    pub(crate) fn get_all_details(
        &self,
        caller: CallContext,
        input: &GetPageInput,
    ) -> ServiceResult<Vec<RegistrationDetails>> {
        caller.must_be_named_principal(PRINCIPAL_NAME_STATE_EXPORTER)?;
        input.validate()?;
        let items = STATE.with(|s| {
            let store = s.registration_store.borrow();
            store
                .get_registrations()
                .values()
                .skip(input.offset)
                .take(input.limit)
                .map(|registration| registration.into())
                .collect()
        });

        Ok(items)
    }

    pub(crate) fn get_owner(&self, name: &str) -> ServiceResult<Principal> {
        let name = validate_name(name)?;
        STATE.with(|s| {
            let store = s.registration_store.borrow();
            let registration = store.get_registration(&name);
            if registration.is_none() {
                return Err(NamingError::RegistrationNotFound);
            }
            Ok(registration.unwrap().get_owner().to_owned())
        })
    }

    pub(crate) fn get_name_expires(&self, name: &str) -> ServiceResult<u64> {
        let name = validate_name(name)?;
        STATE.with(|s| {
            let store = s.registration_store.borrow();
            let registration = store.get_registration(&name);
            if let Some(registration) = registration {
                return Ok(registration.get_expired_at() / 1_000_000);
            }
            Err(NamingError::RegistrationNotFound)
        })
    }

    pub fn validate_quota(
        &self,
        name: &FirstLevelName,
        owner: &AuthPrincipal,
        quota_type: &QuotaType,
        quota_required: u32,
    ) -> Result<(), String> {
        let first = name.0.get_current_level().unwrap();
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
        STATE.with(|s| {
            let user_quota_manager = s.user_quota_store.borrow();
            let quota = user_quota_manager
                .get_quota(owner, &quota_type)
                .unwrap_or(0);
            if quota < quota_required {
                return Err(format!("User has no quota for {}", quota_type));
            }
            Ok(())
        })
    }

    pub async fn register_from_gateway(
        &mut self,
        caller: &Principal,
        name: &str,
        owner: Principal,
        now: TimeInNs,
    ) -> ServiceResult<bool> {
        let owner = must_not_anonymous(&owner)?;
        must_be_named_canister(*caller, CanisterNames::RegistrarControlGateway)?;
        let years = 1;
        let admin_import = true;
        let quota_owner = &AuthPrincipal(get_named_get_canister_id(
            CanisterNames::RegistrarControlGateway,
        ));
        let quota_type = QuotaType::LenGte(1);
        self.register_with_quota_core(
            RegisterCoreContext::new(name.to_string(), owner, years, now, admin_import),
            quota_owner,
            quota_type,
        )
        .await
    }

    pub async fn import_registrations(
        &self,
        call_context: CallContext,
        request: ImportNameRegistrationRequest,
    ) -> ServiceResult<bool> {
        call_context.must_be_system_owner()?;
        for item in request.items {
            let context = RegisterCoreContext::new(
                item.name.clone(),
                must_not_anonymous(&item.owner)?,
                item.years,
                call_context.now,
                true,
            );
            if context.validate().is_ok() {
                let import_result = self.register_core(context).await;
                if let Err(e) = import_result {
                    error!("Failed to import registration: {:?}", e);
                } else {
                    info!("Imported registration: {}", item.name);
                }
            } else {
                error!("Failed to validate registration: {}", item.name);
            }
        }
        Ok(true)
    }

    async fn register_core(&self, context: RegisterCoreContext) -> ServiceResult<bool> {
        let first_level_name = context.validate()?;
        let RegisterCoreContext {
            name,
            owner,
            years,
            now,
            ..
        } = context;

        let expired_at = get_expired_at(years, now);
        let resolver = get_named_get_canister_id(CanisterNames::Resolver);
        let registration = Registration::new(owner.0, name.clone(), expired_at.0, now.0);
        trace!("registering {:?}", registration);
        let api_result = self
            .registry_api
            .set_subdomain_owner(
                first_level_name.0.get_current_level().unwrap().clone(),
                NAMING_TOP_LABEL.to_string(),
                owner.0,
                DEFAULT_TTL,
                resolver,
            )
            .await;
        if api_result.is_ok() {
            trace!("registered success from registry {:?}", registration);
            let own_registration_count = STATE.with(|s| {
                let mut store = s.registration_store.borrow_mut();
                store.add_registration(registration.clone());
                let mut token_index_store = s.token_index_store.borrow_mut();

                match token_index_store.try_add_registration_name(&registration.get_name()) {
                    Ok(token_index) => {
                        trace!(
                            "The index value of the registered name is : {}",
                            token_index.get_value()
                        );
                    }
                    Err(e) => {
                        error!("failed to register success from token index {:?}", e);
                    }
                }
                store.get_user_own_registration_count(&owner.0)
            });
            MERTRICS_COUNTER.with(|c| {
                let mut counter = c.borrow_mut();
                counter.push_registration(registration.clone());
            });
            let _ = self
                .set_record_value(name, &owner.0, own_registration_count)
                .await;
            Ok(true)
        } else {
            Err(NamingError::RemoteError(api_result.err().unwrap()))
        }
    }

    async fn set_record_value(
        &self,
        name: String,
        owner: &Principal,
        own_registration_count: usize,
    ) -> ServiceResult<()> {
        let mut resolver_map = HashMap::new();
        resolver_map.insert(RESOLVER_KEY_ICP_PRINCIPAL.to_string(), owner.to_text());
        resolver_map.insert(
            RESOLVER_KEY_ICP_ACCOUNT_ID.to_string(),
            AccountIdentifier::new(owner.clone(), None).to_hex(),
        );
        if own_registration_count == 1 {
            trace!("user: {} only one registration ", owner);
            resolver_map.insert(
                RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL.to_string(),
                owner.to_text(),
            );
        }
        let api_resolver_result = self.resolver_api.set_record_value(name, resolver_map).await;
        match api_resolver_result {
            Ok(value) => {
                info!("set_record_value api result: {:?}", value);
                Ok(())
            }
            Err(e) => {
                error!("set_record_value api result: {:?}", e);
                Err(NamingError::RemoteError(e))
            }
        }
    }

    pub async fn register_with_quota(
        &mut self,
        name: String,
        owner: Principal,
        years: u32,
        now: TimeInNs,
        quota_owner: &Principal,
        quota_type: QuotaType,
        admin_import: bool,
    ) -> ServiceResult<bool> {
        let owner = must_not_anonymous(&owner)?;
        let quota_owner = must_not_anonymous(quota_owner)?;
        self.register_with_quota_core(
            RegisterCoreContext::new(name, owner, years, now, admin_import),
            &quota_owner,
            quota_type,
        )
        .await
    }

    async fn register_with_quota_core(
        &mut self,
        context: RegisterCoreContext,
        quota_owner: &AuthPrincipal,
        quota_type: QuotaType,
    ) -> ServiceResult<bool> {
        let name_result = context.validate()?;
        // validate quota
        let years = context.years;
        let quota_result = self.validate_quota(&name_result, quota_owner, &quota_type, years);
        if quota_result.is_err() {
            return Err(NamingError::InvalidName {
                reason: quota_result.err().unwrap(),
            });
        }

        // update quota before await in case of concurrent register
        STATE.with(|s| {
            let mut user_quota_manager = s.user_quota_store.borrow_mut();
            user_quota_manager.sub_quota(&quota_owner, &quota_type, years)
        })?;

        let result = self.register_core(context).await;

        if result.is_ok() {
            Ok(true)
        } else {
            // rollback quota
            STATE.with(|s| {
                let mut user_quota_manager = s.user_quota_store.borrow_mut();
                user_quota_manager.add_quota(*quota_owner, quota_type, years);
            });
            Err(result.err().unwrap())
        }
    }

    pub fn available(&self, name: &str) -> ServiceResult<FirstLevelName> {
        let result = validate_name(&name)?;

        // check reserved names
        if RESERVED_NAMES.contains(&result.0.get_current_level().unwrap().as_str()) {
            return Err(NamingError::RegistrationHasBeenTaken);
        }

        STATE.with(|s| {
            let store = s.registration_store.borrow();
            let registration = store.get_registration(&result);
            if registration.is_some() {
                return Err(NamingError::RegistrationHasBeenTaken);
            }
            Ok(result)
        })
    }

    pub fn clean_expired(&mut self, _now_in_ms: u64) -> ServiceResult<()> {
        todo!("clean up")
    }

    pub async fn register_with_payment(
        &self,
        call_context: CallContext,
        request: RegisterNameWithPaymentRequest,
    ) -> ServiceResult<RegistrationDetails> {
        // check
        let caller = call_context.must_not_anonymous()?;
        let name_result = self.available(request.name.as_str())?;
        validate_year(request.years)?;
        let name_len = name_result.get_name_len();
        let length_limit = 6;
        if name_len < length_limit {
            return Err(NamingError::InvalidName {
                reason: format!(
                    "the name need to be at least {} characters long",
                    length_limit,
                ),
            });
        }
        let years = request.years;
        let quota_type_len = name_result.0.get_quota_type_len();
        let amount = self.get_name_price(years, quota_type_len).await?;

        // validate request.approve_price is within the range of register_price 5%
        if request.approve_amount < amount * 95 / 100 {
            debug!(
                "register_with_payment: approve_amount is too low: {} < {}",
                request.approve_amount,
                amount * 95 / 100
            );
            return Err(NamingError::InvalidApproveAmount);
        }

        let result = self
            .token_service
            .transfer_from(
                caller.0.to_text().as_str(),
                DICP_RECEIVER.deref(),
                request.approve_amount.clone(),
                call_context.now,
            )
            .await;
        if let Err(e) = result {
            error!("error transferring: {:?}", e);
            return Err(e);
        }
        let local_tx_id = result.unwrap();
        let context = RegisterCoreContext::new(
            name_result.0.get_name().to_string(),
            caller.clone(),
            years,
            call_context.now,
            false,
        );
        let registration_result = self.register_core(context).await;
        if registration_result.is_ok() {
            info!(
                "registered success, call_context: {:?}, request: {:?}",
                call_context, request
            );
            self.token_service.complete_transaction(local_tx_id);
            self.get_details(name_result.0.get_name())
        } else {
            error!(
                "registered failed, call_context: {:?}, request: {:?}",
                call_context, request
            );
            let _ = self.token_service.refund(local_tx_id).await;
            Err(registration_result.err().unwrap())
        }
    }

    async fn get_name_price(&self, years: u32, quota_type_len: u8) -> ServiceResult<u64> {
        let response = self
            .cycles_minting_api
            .get_icp_xdr_conversion_rate()
            .await?;
        let icp_xdr_conversion_rate = response.data.xdr_permyriad_per_icp;
        assert!(icp_xdr_conversion_rate > 0);
        let price_per_year = get_price_in_icp_e8s(quota_type_len, icp_xdr_conversion_rate);
        let amount = price_per_year * years as u64;
        debug!(
            "price_per_year: {}, amount: {}, icp_xdr_conversion_rate: {}",
            price_per_year, amount, icp_xdr_conversion_rate
        );
        Ok(amount)
    }

    pub fn batch_add_quota(
        &mut self,
        call_context: CallContext,
        request: BatchAddQuotaRequest,
    ) -> ServiceResult<bool> {
        let caller = call_context.must_be_system_owner()?;
        for item in request.items.iter() {
            let quota_type = QuotaType::from_str(&item.quota_type).unwrap();
            self.add_quota(&caller.0, item.owner, quota_type, item.diff)?;
        }
        Ok(true)
    }

    pub fn add_quota(
        &mut self,
        caller: &Principal,
        quota_owner: Principal,
        quota_type: QuotaType,
        diff: u32,
    ) -> ServiceResult<bool> {
        must_be_system_owner(caller)?;
        let quota_owner = must_not_anonymous(&quota_owner)?;
        STATE.with(|s| {
            let mut user_quota_manager = s.user_quota_store.borrow_mut();
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
    ) -> ServiceResult<bool> {
        must_be_system_owner(caller)?;
        let quota_owner = must_not_anonymous(&quota_owner)?;
        STATE.with(|s| {
            let mut user_quota_manager = s.user_quota_store.borrow_mut();
            user_quota_manager.sub_quota(&quota_owner, &quota_type, diff)
        })?;
        Ok(true)
    }

    pub fn get_quota(
        &self,
        caller: &Principal,
        quota_owner: Principal,
        quota_type: QuotaType,
    ) -> ServiceResult<u32> {
        must_not_anonymous(caller)?;
        STATE.with(|s| {
            let user_quota_manager = s.user_quota_store.borrow();
            let target_user = must_not_anonymous(&quota_owner)?;
            Ok(user_quota_manager
                .get_quota(&target_user, &quota_type)
                .unwrap_or(0))
        })
    }

    pub async fn get_price_table(&self) -> ServiceResult<PriceTable> {
        let response = self
            .cycles_minting_api
            .get_icp_xdr_conversion_rate()
            .await?;
        let icp_xdr_conversion_rate = response.data.xdr_permyriad_per_icp;
        assert!(icp_xdr_conversion_rate > 0);

        let mut items = vec![];
        for x in 1..=7 {
            items.push(PriceTableItem {
                len: x,
                price_in_xdr_permyriad: get_price_in_xdr_permyriad(x).to_u64().unwrap(),
                price_in_icp_e8s: get_price_in_icp_e8s(x, icp_xdr_conversion_rate),
            });
        }
        Ok(PriceTable {
            items,
            icp_xdr_conversion_rate,
        })
    }

    pub fn import_quota(
        &self,
        caller: &Principal,
        request: ImportQuotaRequest,
    ) -> ServiceResult<ImportQuotaStatus> {
        must_be_named_canister(*caller, CanisterNames::RegistrarControlGateway)?;
        let result = STATE.with(|s| {
            let store = s.quota_import_store.borrow();
            store.verify_hash(&request.hash)
        });
        if result.is_err() {
            return Ok(ImportQuotaStatus::AlreadyExists);
        }

        let items = request.items;
        // apply items and save hashes
        STATE.with(|s| {
            let mut store = s.user_quota_store.borrow_mut();
            for item in items.iter() {
                let quota_to = must_not_anonymous(&item.owner)?;
                store.add_quota(
                    quota_to,
                    QuotaType::from_str(item.quota_type.as_str()).unwrap(),
                    item.diff,
                );
            }

            let hash = request.hash;
            let mut import_quota_store = s.quota_import_store.borrow_mut();
            info!("file imported, save hashes: {}", hex::encode(&hash));
            import_quota_store.add_imported_file_hash(hash);
            Ok(ImportQuotaStatus::Ok)
        })
    }

    fn is_name_owner(&self, name: &FirstLevelName, caller: &Principal) -> ServiceResult<Principal> {
        STATE.with(|s| {
            let store = s.registration_store.borrow();
            let registration = store.get_registration(name);
            if registration.is_none() {
                return Err(NamingError::RegistrationNotFound);
            }
            let registration = registration.unwrap();
            let owner = registration.get_owner();

            if !owner.eq(caller) {
                return Err(NamingError::PermissionDenied);
            }

            Ok(owner)
        })
    }

    pub async fn reclaim_name(&self, name: &str, caller: &Principal) -> ServiceResult<bool> {
        let name = validate_name(&name)?;
        must_not_anonymous(caller)?;
        let registration_owner = self.is_name_owner(&name, caller)?;
        debug!("reclaim name: {} to user {}", name, &registration_owner);

        let resolver = get_named_get_canister_id(CanisterNames::Resolver);
        try_lock_name(&name)?;
        let reclaim_result = self
            .registry_api
            .reclaim_name(name.to_string(), registration_owner, resolver)
            .await;
        unlock_name(&name);

        let result = match reclaim_result {
            Ok(result) => {
                info!(
                    "reclaim name: {} to user {} success",
                    name, &registration_owner
                );
                Ok(result)
            }
            Err(e) => {
                error!(
                    "reclaim name: {} to user {} failed: {}",
                    name, &registration_owner, e.message
                );
                Err(NamingError::RemoteError(e))
            }
        };
        result
    }

    async fn transfer_core(
        &self,
        name: &FirstLevelName,
        new_owner: &Principal,
    ) -> ServiceResult<bool> {
        STATE.with(|s| {
            let store = s.registration_store.borrow();
            if !store.has_registration(name) {
                return Err(NamingError::RegistrationNotFound);
            }
            Ok(())
        })?;
        try_lock_name(&name)?;
        let registry_result = self
            .registry_api
            .transfer(
                name.to_string(),
                *new_owner,
                get_named_get_canister_id(CanisterNames::Resolver),
            )
            .await;
        unlock_name(&name);
        registry_result?;

        STATE.with(|s| {
            let mut store = s.registration_store.borrow_mut();
            store.transfer_registration(name.to_string(), *new_owner);

            let mut store = s.registration_approval_store.borrow_mut();
            store.remove_approval(name);

            info!("transfer name: {} to user {}", name, &new_owner);
            Ok(true)
        })
    }

    pub(crate) async fn transfer(
        &self,
        name: &str,
        caller: &Principal,
        new_owner: Principal,
    ) -> ServiceResult<bool> {
        let name = validate_name(&name)?;
        must_not_anonymous(caller)?;
        must_not_anonymous(&new_owner)?;
        self.is_name_owner(&name, caller)?;
        assert_ne!(caller, &new_owner);

        self.transfer_core(&name, &new_owner).await
    }

    // TODO: remove this function when all assignment is done
    pub async fn transfer_by_admin(
        &self,
        name: &str,
        caller: &Principal,
        new_owner: Principal,
    ) -> ServiceResult<bool> {
        must_be_system_owner(caller)?;
        let name = validate_name(name)?;
        assert!(RESERVED_NAMES
            .iter()
            .any(|n| n == name.0.get_current_level().unwrap()));
        must_not_anonymous(&new_owner)?;

        self.transfer_core(&name, &new_owner).await
    }

    pub fn approve(
        &self,
        caller: &Principal,
        now: u64,
        name: &str,
        to: Principal,
    ) -> ServiceResult<bool> {
        let name = validate_name(name)?;
        must_not_anonymous(caller)?;
        let _ = self.is_name_owner(&name, caller)?;
        assert_ne!(caller, &to);

        STATE.with(|s| {
            let mut store = s.registration_approval_store.borrow_mut();
            store.set_approval(&name, &to, now);
            Ok(true)
        })
    }

    pub async fn transfer_from(
        &self,
        caller: &Principal,
        name: &str,
        to: Option<AuthPrincipal>,
    ) -> ServiceResult<bool> {
        let name = validate_name(name)?;
        must_not_anonymous(caller)?;
        STATE.with(|s| {
            let store = s.registration_approval_store.borrow_mut();
            if !store.is_approved_to(&name, caller) {
                return Err(NamingError::PermissionDenied);
            }

            Ok(())
        })?;
        match to {
            Some(to) => self.transfer_core(&name, &to.0).await,
            None => self.transfer_core(&name, &caller).await,
        }
    }

    pub fn transfer_from_quota(
        &self,
        caller: &Principal,
        from: Principal,
        to: Principal,
        quota_type: QuotaType,
        diff: u32,
    ) -> ServiceResult<bool> {
        if must_be_system_owner(caller).is_err() {
            must_be_in_named_canister(
                *caller,
                &[CanisterNames::MysteryBox, CanisterNames::NamingMarketplace],
            )?;
        } else {
            debug!("transfer_from_quota: caller is system owner");
        }

        let to = must_not_anonymous(&to)?;
        let from = must_not_anonymous(&from)?;
        assert!(diff > 0);

        STATE.with(|s| {
            let mut store = s.user_quota_store.borrow_mut();
            let quota_count = store.get_quota(&from, &quota_type).unwrap_or(0);
            if quota_count < diff {
                return Err(NamingError::InsufficientQuota);
            }

            store.sub_quota(&from, &quota_type, diff)?;
            store.add_quota(to, quota_type, diff);
            info!(
                "transfer quota: {} from user {} to user {}, diff: {}",
                quota_type, &from, &to, diff
            );
            Ok(true)
        })
    }

    pub fn transfer_quota(
        &self,
        caller: Principal,
        details: TransferQuotaDetails,
    ) -> ServiceResult<bool> {
        let caller = must_not_anonymous(&caller)?;
        must_not_anonymous(&details.to)?;
        assert_ne!(caller.0, details.to);

        STATE.with(|s| {
            let mut store = s.user_quota_store.borrow_mut();
            store.transfer_quota(&caller, &details)?;
            Ok(true)
        })
    }

    pub fn batch_transfer_quota(
        &self,
        caller: Principal,
        request: BatchTransferRequest,
    ) -> ServiceResult<bool> {
        let caller = must_not_anonymous(&caller)?;
        for item in request.items.iter() {
            must_not_anonymous(&item.to)?;
            assert_ne!(caller.0, item.to);
        }

        STATE.with(|s| {
            let mut store = s.user_quota_store.borrow_mut();
            store.batch_transfer_quota(caller, request.items.as_slice())?;
            Ok(true)
        })
    }

    pub fn unlock_names(&self, caller: &Principal, names: Vec<&str>) -> ServiceResult<bool> {
        must_be_system_owner(caller)?;
        NAME_LOCKER.with(|locker| {
            let mut locker = locker.borrow_mut();
            for name in names {
                locker.unlock(name);
            }
            Ok(true)
        })
    }

    pub fn get_last_registrations(
        &self,
        caller: &Principal,
    ) -> ServiceResult<Vec<RegistrationDetails>> {
        must_be_named_principal(caller, PRINCIPAL_NAME_TIMER_TRIGGER)?;
        MERTRICS_COUNTER.with(|counter| {
            let counter = counter.borrow();
            let mut result = Vec::new();
            for details in counter.last_registrations.iter() {
                result.push(details.into());
            }
            Ok(result)
        })
    }

    pub fn get_public_resolver(&self) -> String {
        get_named_get_canister_id(CanisterNames::Resolver).to_text()
    }

    pub async fn renew_name(
        &self,
        caller: Principal,
        now: TimeInNs,
        request: RenewNameRequest,
    ) -> ServiceResult<bool> {
        assert!(request.years > 0);
        assert!(request.approve_amount > 0);
        must_not_anonymous(&caller)?;
        let first_level_name = validate_name(&request.name)?;
        let renew_price = self
            .get_name_price(request.years, first_level_name.0.get_quota_type_len())
            .await?;

        // validate request.approve_price is within the range of renew_price 10%
        let approve_amount = request.approve_amount;
        if approve_amount < renew_price * 95 / 100 {
            return Err(NamingError::InvalidApproveAmount);
        }

        let new_expired_at = STATE.with(|s| {
            let registration_store = s.registration_store.borrow();
            if let Some(registration) = registration_store.get_registration(&first_level_name) {
                if !registration.is_owner(&caller) {
                    return Err(NamingError::InvalidOwner);
                }
                let new_expired_at =
                    get_expired_at(request.years, TimeInNs(registration.get_expired_at()));
                if new_expired_at > get_expired_at(NAMING_MAX_REGISTRATION_YEAR, now) {
                    return Err(NamingError::RenewalYearsError {
                        years: NAMING_MAX_REGISTRATION_YEAR,
                    });
                }
                Ok(new_expired_at)
            } else {
                Err(NamingError::InvalidName {
                    reason: "name not registered".to_string(),
                })
            }
        })?;

        let result = self
            .token_service
            .transfer_from(
                caller.to_text().as_str(),
                DICP_RECEIVER.deref(),
                Nat::from(request.approve_amount),
                now,
            )
            .await;
        if let Err(e) = result {
            error!("error transferring: {:?}", e);
            return Err(e);
        }

        STATE.with(|s| {
            let mut registration_store = s.registration_store.borrow_mut();
            registration_store.update_expired_at(&first_level_name, new_expired_at.0);
        });
        let local_tx_id = result.unwrap();
        self.token_service.complete_transaction(local_tx_id);
        Ok(true)
    }

    pub fn get_name_status(&self, name: &str) -> ServiceResult<NameStatus> {
        let name = validate_name(name)?;
        if let Some(status) = STATE.with(|s| {
            let registration_store = s.registration_store.borrow();
            if let Some(registration) = registration_store.get_registration(&name) {
                return Some(NameStatus {
                    registered: true,
                    available: false,
                    kept: false,
                    details: Some(registration.into()),
                });
            }
            return None;
        }) {
            return Ok(status);
        }

        // check reserved names
        if RESERVED_NAMES.contains(&name.0.get_current_level().unwrap().as_str()) {
            return Ok(NameStatus {
                kept: true,
                registered: false,
                available: false,
                details: None,
            });
        }

        return Ok(NameStatus {
            registered: false,
            available: true,
            kept: false,
            details: None,
        });
    }

    pub(crate) fn get_registry(&self, now: u64) -> Vec<(u32, String)> {
        let mut list = STATE.with(|s| {
            let token_index_store = s.token_index_store.borrow();
            let registration_store = s.registration_store.borrow();
            let query = RegistrationNameQueryContext::new(&token_index_store, &registration_store);
            let unexpired_registration_names = query
                .get_all_unexpired_registrations(now)
                .iter()
                .map(|registration_agg| {
                    (
                        registration_agg.get_index().get_value(),
                        registration_agg.get_name(),
                    )
                })
                .collect::<Vec<_>>();
            unexpired_registration_names
        });
        list.sort_by(|a, b| a.0.cmp(&b.0));

        list
    }

    pub(crate) fn get_tokens(&self, now: u64) -> Vec<(u32, Metadata)> {
        let mut list = STATE.with(|s| {
            let token_index_store = s.token_index_store.borrow();
            let registration_store = s.registration_store.borrow();
            let query = RegistrationNameQueryContext::new(&token_index_store, &registration_store);
            let unexpired_registration_names = query
                .get_all_unexpired_registrations(now)
                .iter()
                .map(|registration_agg| {
                    (
                        registration_agg.get_index().get_value(),
                        Metadata::NonFungible({
                            let metadata = NonFungible {
                                metadata: registration_agg.get_metadata(),
                            };
                            metadata
                        }),
                    )
                })
                .collect::<Vec<_>>();
            unexpired_registration_names
        });
        list.sort_by(|a, b| a.0.cmp(&b.0));

        list
    }

    pub(crate) fn metadata(&self, token: &TokenIdentifier, now: u64) -> NFTServiceResult<Metadata> {
        let registration_name = STATE.with(|s| {
            let token_index_store = s.token_index_store.borrow();
            let registration_store = s.registration_store.borrow();
            let query = RegistrationNameQueryContext::new(&token_index_store, &registration_store);
            query.get_unexpired_registration(token, now)
        })?;
        return Ok(Metadata::NonFungible({
            let metadata = NonFungible {
                metadata: registration_name.get_metadata(),
            };
            metadata
        }));
    }

    pub(crate) fn supply(&self) -> NFTServiceResult<u128> {
        STATE.with(|s| {
            let token_index_store = s.token_index_store.borrow();
            Ok(token_index_store.get_current_token_index().get_value() as u128)
        })
    }

    pub(crate) fn bearer(&self, token: &TokenIdentifier, now: u64) -> NFTServiceResult<String> {
        let registration = STATE.with(|s| {
            let token_index_store = s.token_index_store.borrow();
            let registration_store = s.registration_store.borrow();
            let query = RegistrationNameQueryContext::new(&token_index_store, &registration_store);
            query.get_unexpired_registration(token, now)
        })?;
        Ok(registration.get_owner().to_text())
    }

    pub(crate) fn import_token_id_from_registration(
        &self,
        call_context: &CallContext,
    ) -> ServiceResult<usize> {
        let _ = call_context.must_be_system_owner()?;
        STATE.with(|s| {
            let registration_store = s.registration_store.borrow();
            let mut token_index_store = s.token_index_store.borrow_mut();
            let registrations = registration_store
                .get_registrations()
                .values()
                .map(|registration| registration.get_name())
                .collect::<Vec<_>>();
            let success_count = token_index_store.import_from_registration_store(&registrations);
            Ok(success_count)
        })
    }
    pub(crate) fn ext_approve(
        &self,
        call_context: &CallContext,
        spender: Principal,
        token: &TokenIdentifier,
        now: u64,
    ) -> NFTServiceResult<()> {
        let reggistration_agg = STATE.with(|s| {
            let token_index_store = s.token_index_store.borrow();
            let registration_store = s.registration_store.borrow();
            let query = RegistrationNameQueryContext::new(&token_index_store, &registration_store);
            query.get_unexpired_registration(token, now)
        })?;
        let _ = self.approve(
            &call_context.caller,
            call_context.now.0,
            reggistration_agg.get_name().as_str(),
            spender,
        );
        Ok(())
    }

    pub(crate) fn allowance(
        &self,
        owner: &common::nft::User,
        spender: &Principal,
        token: &TokenIdentifier,
        now: u64,
    ) -> NFTServiceResult<u128> {
        let owner = owner.get_principal()?;
        let registration = STATE.with(|s| {
            let token_index_store = s.token_index_store.borrow();
            let registration_store = s.registration_store.borrow();
            let query = RegistrationNameQueryContext::new(&token_index_store, &registration_store);
            query.get_unexpired_registration(token, now)
        })?;
        if !registration.is_owner(&owner) {
            return Err(NamingError::InvalidOwner.into());
        }
        STATE.with(|s| {
            let approve_store = s.registration_approval_store.borrow();
            if approve_store.is_approved_to(&registration.get_name().into(), &spender) {
                return Ok(1);
            }
            Ok(0)
        })
    }
    pub(crate) async fn ext_transfer(
        &self,
        call_context: &CallContext,
        from: &common::nft::User,
        to: &common::nft::User,
        token: &TokenIdentifier,
        now: u64,
    ) -> NFTTransferServiceResult<u128> {
        let from = from.get_principal()?;
        let to = to.get_principal()?;
        let registration = STATE.with(|s| {
            let token_index_store = s.token_index_store.borrow();
            let registration_store = s.registration_store.borrow();
            let query = RegistrationNameQueryContext::new(&token_index_store, &registration_store);
            query.get_unexpired_registration(token, now)
        })?;
        if !registration.is_owner(&from) {
            return Err(NamingError::InvalidOwner.into());
        }
        if !registration.is_owner(&call_context.caller) {
            let to_auth = must_not_anonymous(&to)?;
            let transfer_result = self
                .transfer_from(
                    &call_context.caller,
                    registration.get_name().as_str(),
                    Some(to_auth),
                )
                .await;
            match transfer_result {
                Ok(value) => Ok(value as u128),
                Err(e) => Err(e.into()),
            }
        } else {
            let transfer_result = self
                .transfer(registration.get_name().as_str(), &from, to)
                .await;
            match transfer_result {
                Ok(value) => Ok(value as u128),
                Err(e) => Err(e.into()),
            }
        }
    }

    pub(crate) fn get_token_details_by_names(
        &self,
        names: &Vec<String>,
        now: u64,
    ) -> HashMap<String, Option<(u32, String)>> {
        STATE.with(|s| {
            let token_index_store = s.token_index_store.borrow();
            let registration_store = s.registration_store.borrow();
            let query = RegistrationNameQueryContext::new(&token_index_store, &registration_store);
            let list = query.get_unexpired_registration_agg_by_names(names, now);
            list.iter()
                .map(|item| {
                    return match item {
                        GetUnexpiredRegistrationAggByNamesResult::Valid(registration_agg) => (
                            registration_agg.get_name(),
                            Some((
                                registration_agg.get_index().get_value(),
                                registration_agg.get_id().to_owned(),
                            )),
                        ),
                        GetUnexpiredRegistrationAggByNamesResult::NotFound(name) => {
                            (name.to_owned(), None)
                        }
                    };
                })
                .collect()
        })
    }
}

fn get_price_in_xdr_permyriad(len: u8) -> BigUint {
    assert!(len > 0);
    match len {
        1 => BigUint::from(35400u32),
        2 => BigUint::from(32200u32),
        3 => BigUint::from(29200u32),
        4 => BigUint::from(26600u32),
        5 => BigUint::from(24200u32),
        6 => BigUint::from(22000u32),
        _ => BigUint::from(20000u32),
    }
}

fn get_price_in_icp_e8s(len: u8, xdr_permyriad_per_icp: u64) -> u64 {
    // price_in_icp = get_price_in_xdr_permyriad / xdr_permyriad_per_icp
    // it is needed change to icp_e8s, and price_in_icp_e8s = price_in_icp * 10^8
    // we want to keep 4 digits after decimal point, so we need to multiply 10^4 for twice other than 10^8 for once
    let xdr_permyriad = get_price_in_xdr_permyriad(len) * BigUint::from(10_000u32);
    let e8s = xdr_permyriad / BigUint::from(xdr_permyriad_per_icp) * BigUint::from(10_000u32);
    let result = e8s.to_u64().unwrap();
    // 0.01 icp = 10^6
    assert!(result > 1_000_000);
    result
}

#[derive(CandidType)]
pub struct PriceTableItem {
    pub len: u8,
    pub price_in_icp_e8s: u64,
    pub price_in_xdr_permyriad: u64,
}

#[derive(CandidType)]
pub struct PriceTable {
    pub icp_xdr_conversion_rate: u64,
    pub items: Vec<PriceTableItem>,
}

fn validate_name(name: &str) -> ServiceResult<FirstLevelName> {
    assert!(!name.is_empty());
    let name = normalize_name(name);
    let result = NameParseResult::parse(&name);
    if result.get_level_count() != 2 {
        return Err(NamingError::InvalidName {
            reason: "it must be second level name".to_string(),
        });
    }
    if result.get_top_level().unwrap() != NAMING_TOP_LABEL {
        return Err(NamingError::InvalidName {
            reason: format!("top level of name must be {}", NAMING_TOP_LABEL),
        });
    }
    let first = result.get_current_level().unwrap();
    if first.len() > 63 {
        return Err(NamingError::InvalidName {
            reason: "second level name must be less than 64 characters".to_string(),
        });
    }

    if !first.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(NamingError::InvalidName {
            reason: "name must be alphanumeric or -".to_string(),
        });
    }
    Ok(FirstLevelName(result))
}

fn validate_year(years: u32) -> ServiceResult<()> {
    if years < NAMING_MIN_REGISTRATION_YEAR || years > NAMING_MAX_REGISTRATION_YEAR {
        return Err(NamingError::YearsRangeError {
            min: NAMING_MIN_REGISTRATION_YEAR,
            max: NAMING_MAX_REGISTRATION_YEAR,
        });
    }
    Ok(())
}

pub fn get_expired_at(years: u32, now: TimeInNs) -> TimeInNs {
    let now_time = OffsetDateTime::from_unix_timestamp_nanos(now.0 as i128).unwrap();
    // remove ms and ns
    let now_time = now_time
        .replace_time(Time::from_hms(0, 0, 0).unwrap())
        .replace_nanosecond(0)
        .unwrap();
    // add years
    let expired_at = now_time
        .replace_year(now_time.year() + years as i32)
        .unwrap();
    let expired_at = expired_at.unix_timestamp_nanos() as u64;
    TimeInNs(expired_at)
}

struct RegisterCoreContext {
    pub name: String,
    pub owner: AuthPrincipal,
    pub years: u32,
    pub now: TimeInNs,
    pub admin_import: bool,
}

impl RegisterCoreContext {
    pub fn new(
        name: String,
        owner: AuthPrincipal,
        years: u32,
        now: TimeInNs,
        admin_import: bool,
    ) -> Self {
        RegisterCoreContext {
            name,
            owner,
            years,
            now,
            admin_import,
        }
    }

    pub fn validate(&self) -> ServiceResult<FirstLevelName> {
        // validate name
        let first_level_name = validate_name(&self.name)?;

        // validate year
        validate_year(self.years)?;

        // check reservation if not admin import
        if !self.admin_import {
            // check reserved names
            if RESERVED_NAMES.contains(&first_level_name.0.get_current_level().unwrap().as_str()) {
                return Err(NamingError::RegistrationHasBeenTaken);
            }
        }

        STATE.with(|s| {
            let store = s.registration_store.borrow();
            if store.has_registration(&first_level_name) {
                return Err(NamingError::RegistrationHasBeenTaken);
            }
            Ok(())
        })?;
        Ok(first_level_name)
    }
}

pub struct RegistrationNameQueryContext<'a> {
    token_index_store: &'a TokenIndexStore,
    registration_store: &'a RegistrationStore,
}

impl<'a> RegistrationNameQueryContext<'a> {
    pub fn new(
        token_index_store: &'a TokenIndexStore,
        registration_store: &'a RegistrationStore,
    ) -> Self {
        Self {
            token_index_store: token_index_store,
            registration_store: registration_store,
        }
    }

    pub fn get_all_unexpired_registrations(&self, now: u64) -> Vec<UnexpiredRegistrationAggDto> {
        let registration_names = self.token_index_store.get_registrations();
        let mut valid_registration_names = Vec::new();

        registration_names.iter().for_each(|registration_name_ref| {
            let registration_name = registration_name_ref.borrow();
            let registration_result =
                self.get_unexpired_registration_by_name(&registration_name.get_name(), now);
            match registration_result {
                Ok(registration) => {
                    let id = encode_token_id(
                        common::token_identifier::CanisterId(get_named_get_canister_id(
                            CanisterNames::Registrar,
                        )),
                        registration_name.get_index(),
                    );
                    valid_registration_names.push(UnexpiredRegistrationAggDto::new(
                        &registration,
                        &registration_name,
                        &id,
                    ));
                }
                Err(_) => {
                    // ignore error
                }
            }
        });
        valid_registration_names
    }
    pub fn get_unexpired_registration_agg_by_names(
        &self,
        names: &Vec<String>,
        now: u64,
    ) -> Vec<GetUnexpiredRegistrationAggByNamesResult> {
        names
            .iter()
            .map(|name| {
                let registration_name_result = self.get_registration_name_by_name(name);
                let registration_result = self.get_unexpired_registration_by_name(name, now);
                return match (registration_result, registration_name_result) {
                    (Ok(registration), Ok(registration_name)) => {
                        let id = encode_token_id(
                            common::token_identifier::CanisterId(get_named_get_canister_id(
                                CanisterNames::Registrar,
                            )),
                            registration_name.get_index(),
                        );
                        GetUnexpiredRegistrationAggByNamesResult::Valid(
                            UnexpiredRegistrationAggDto::new(
                                &registration,
                                &registration_name,
                                &id,
                            ),
                        )
                    }
                    _ => {
                        // ignore error
                        GetUnexpiredRegistrationAggByNamesResult::NotFound(name.to_owned())
                    }
                };
            })
            .collect()
    }

    pub fn get_unexpired_registration(
        &self,
        token_id: &TokenIdentifier,
        now: u64,
    ) -> NFTServiceResult<UnexpiredRegistrationAggDto> {
        let registration_name = self.get_registration_name_by_token_id(token_id)?;
        let registration =
            self.get_unexpired_registration_by_name(&registration_name.get_name(), now)?;
        let unexpired_registration_agg =
            UnexpiredRegistrationAggDto::new(&registration, &registration_name, token_id);

        Ok(unexpired_registration_agg)
    }

    pub fn get_registration_name_by_token_id(
        &self,
        token_id: &TokenIdentifier,
    ) -> NFTServiceResult<RegistrationName> {
        let index = get_valid_token_index(token_id, CanisterNames::Registrar)?;

        let registration_name = self.token_index_store.get_registration(&index);
        if let Some(registration_name) = registration_name {
            return Ok(registration_name.borrow().clone());
        }
        Err(CommonError::InvalidToken(token_id.to_owned()))
    }
    pub fn get_registration_name_by_name(
        &self,
        name: &String,
    ) -> NFTServiceResult<RegistrationName> {
        let registration_name = self.token_index_store.get_registration_by_name(name);
        if let Some(registration_name) = registration_name {
            return Ok(registration_name.borrow().clone());
        }
        Err(NamingError::RegistrationNotFound.into())
    }

    pub fn get_unexpired_registration_by_name(
        &self,
        name: &String,
        now: u64,
    ) -> NFTServiceResult<Registration> {
        let registration = self
            .registration_store
            .get_registration(&name.to_owned().into());
        if let Some(registration) = registration {
            if !registration.is_expired(now) {
                return Ok(registration.to_owned());
            }
        }
        Err(NamingError::RegistrationNotFound.into())
    }
}

pub enum GetUnexpiredRegistrationAggByNamesResult {
    Valid(UnexpiredRegistrationAggDto),
    NotFound(String),
}

#[derive(Debug, Deserialize, CandidType)]
pub struct BatchTransferRequest {
    pub items: Vec<TransferQuotaDetails>,
}

#[derive(Debug, Deserialize, CandidType)]
pub struct RenewNameRequest {
    pub name: String,
    pub years: u32,
    pub approve_amount: u64,
}

#[derive(Debug, Deserialize, CandidType)]
pub struct ImportNameRegistrationItem {
    pub name: String,
    pub owner: Principal,
    pub years: u32,
}

#[derive(Debug, Deserialize, CandidType)]
pub struct ImportNameRegistrationRequest {
    pub items: Vec<ImportNameRegistrationItem>,
}

#[derive(Debug, Deserialize, CandidType)]
pub struct NameStatus {
    pub available: bool,
    pub kept: bool,
    pub registered: bool,
    pub details: Option<RegistrationDetails>,
}

#[cfg(test)]
mod tests;

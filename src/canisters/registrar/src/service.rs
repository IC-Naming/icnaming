use std::cmp::min;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;
use time::{OffsetDateTime, Time};

use candid::{CandidType, Deserialize, Nat, Principal};

use log::{debug, error, info, trace};
use num_bigint::BigUint;
use num_traits::ToPrimitive;

use common::canister_api::ic_impl::{CyclesMintingApi, DICPApi, RegistryApi};
use common::canister_api::{ICyclesMintingApi, IDICPApi, IRegistryApi};
use common::constants::*;
use common::dto::{GetPageInput, GetPageOutput, ImportQuotaRequest, ImportQuotaStatus};
use common::errors::{NamingError, ServiceResult};

use common::named_canister_ids::{get_named_get_canister_id, CanisterNames};
use common::named_principals::PRINCIPAL_NAME_TIMER_TRIGGER;
use common::naming::{normalize_name, FirstLevelName, NameParseResult, NormalizedName};
use common::permissions::{
    is_admin, must_be_in_named_canister, must_be_named_canister, must_be_system_owner,
};
use common::permissions::{must_be_named_principal, must_not_anonymous};
use common::{AuthPrincipal, CallContext, TimeInNs};

use crate::name_locker::{try_lock_name, unlock_name};
use crate::name_order_store::{GetNameOrderResponse, NameOrder};
use crate::registration_store::{Registration, RegistrationDetails, RegistrationDto};
use crate::reserved_list::RESERVED_NAMES;
use crate::state::*;
use crate::token_service::TokenService;

use crate::user_quota_store::{QuotaType, TransferQuotaDetails};

#[derive(Deserialize, CandidType)]
pub struct SubmitOrderRequest {
    pub name: String,
    pub years: u32,
}

#[derive(Deserialize, CandidType)]
pub struct SubmitOrderResponse {
    pub order: GetNameOrderResponse,
}

pub struct RegistrarService {
    pub registry_api: Arc<dyn IRegistryApi>,
    pub cycles_minting_api: Arc<dyn ICyclesMintingApi>,
    pub token_service: TokenService,
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

        let items = STATE.with(|s| {
            let store = s.registration_store.borrow();
            store
                .get_registrations()
                .values()
                .filter(|registration| registration.is_owner(owner))
                .skip(input.offset)
                .take(input.limit)
                .map(|registration| registration.into())
                .collect()
        });

        Ok(GetPageOutput::new(items))
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
        caller: &Principal,
        input: &GetPageInput,
    ) -> ServiceResult<Vec<RegistrationDetails>> {
        must_be_system_owner(caller)?;
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
            let import_result = self
                .register_core(RegisterCoreContext::new(
                    item.name.clone(),
                    must_not_anonymous(&item.owner)?,
                    item.years,
                    call_context.now,
                    true,
                ))
                .await;
            if let Err(e) = import_result {
                error!("Failed to import registration: {:?}", e);
            } else {
                info!("Imported registration: {}", item.name);
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
            STATE.with(|s| {
                let mut store = s.registration_store.borrow_mut();
                store.add_registration(registration.clone());
            });
            MERTRICS_COUNTER.with(|c| {
                let mut counter = c.borrow_mut();
                counter.push_registration(registration.clone());
            });
            Ok(true)
        } else {
            Err(NamingError::RemoteError(api_result.err().unwrap()))
        }
    }

    pub async fn pay_my_order(&self, caller: Principal, now: TimeInNs) -> ServiceResult<bool> {
        must_not_anonymous(&caller)?;
        let order = STATE.with(|s| {
            let name_order_store = s.name_order_store.borrow();
            if let Some(order) = name_order_store.get_order(&caller) {
                return Ok(order.clone());
            }
            Err(NamingError::OrderNotFound)
        })?;
        let first_level_name = validate_name(order.name()).unwrap();

        let amount = order.price_icp_in_e8s();
        if let Err(s) = STATE.with(|s| {
            let mut name_order_store = s.name_order_store.borrow_mut();
            name_order_store.add_handling_name(&first_level_name)
        }) {
            error!("error adding handling name: {}", s);
            return Err(NamingError::Conflict);
        };
        let result = self
            .token_service
            .transfer_from(
                caller.to_text().as_str(),
                DICP_RECEIVER.deref(),
                amount.clone(),
                now,
            )
            .await;
        STATE.with(|s| {
            let mut name_order_store = s.name_order_store.borrow_mut();
            name_order_store
                .remove_handling_name(&first_level_name)
                .unwrap();
        });
        if let Err(e) = result {
            error!("error transferring: {:?}", e);
            return Err(e);
        }
        let local_tx_id = result.unwrap();
        let context: RegisterCoreContext = (&order).into();
        let registration_result = self.register_core(context).await;
        if registration_result.is_ok() {
            info!("registered success: {:?}", order);
            STATE.with(|s| {
                let mut name_order_store = s.name_order_store.borrow_mut();
                name_order_store.remove_name_order(&caller);
            });
            self.token_service.complete_transaction(local_tx_id);
            Ok(true)
        } else {
            error!("registered failed: {:?}", order);
            let _ = self.token_service.refund(local_tx_id).await;
            Err(registration_result.err().unwrap())
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

    pub fn has_pending_order(&self, caller: &Principal) -> ServiceResult<bool> {
        must_not_anonymous(caller)?;
        Ok(STATE.with(|s| {
            let name_order_store = s.name_order_store.borrow();
            name_order_store.has_name_order(caller)
        }))
    }

    pub fn get_pending_order(
        &self,
        caller: &Principal,
    ) -> ServiceResult<Option<GetNameOrderResponse>> {
        must_not_anonymous(caller)?;
        Ok(STATE.with(|s| {
            let name_order_store = s.name_order_store.borrow();
            name_order_store.get_order(caller).map(|order| order.into())
        }))
    }

    pub async fn submit_order(
        &self,
        caller: &Principal,
        now: u64,
        request: SubmitOrderRequest,
    ) -> ServiceResult<SubmitOrderResponse> {
        // check
        must_not_anonymous(caller)?;
        ensure_no_pending_name_order(caller)?;
        let name_result = self.available(request.name.as_str())?;
        validate_year(request.years)?;
        let name_length = name_result.0.get_name_len();
        let length_limit = 6;
        if name_length < length_limit {
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

        ensure_no_pending_name_order(caller)?;

        // place name order
        let get_order_result: GetNameOrderResponse = STATE.with(|s| {
            let mut name_order_store = s.name_order_store.borrow_mut();
            name_order_store.add_name_order(
                caller,
                request.name.as_str(),
                request.years,
                Nat::from(amount),
                now,
            );
            name_order_store.get_order(caller).unwrap().into()
        });

        Ok(SubmitOrderResponse {
            order: get_order_result,
        })
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

    pub fn cancel_order(&self, caller: &Principal, now: u64) -> ServiceResult<bool> {
        must_not_anonymous(caller)?;
        STATE.with(|s| {
            let mut name_order_store = s.name_order_store.borrow_mut();
            let order = name_order_store.get_order(caller);
            if order.is_none() {
                return Err(NamingError::OrderNotFound);
            }
            name_order_store.cancel_name_order(caller);
            Ok(true)
        })?;

        STATE.with(|s| {
            let mut name_order_store = s.name_order_store.borrow_mut();
            name_order_store.remove_name_order(caller);
            Ok(true)
        })
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
            let target_user = if is_admin(caller) {
                quota_owner
            } else {
                *caller
            };
            Ok(user_quota_manager
                .get_quota(&AuthPrincipal(target_user), &quota_type)
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

    pub fn cancel_expired_orders(&self, now: u64) -> ServiceResult<bool> {
        let (need_check_orders, expired_order) = STATE.with(|s| {
            let store = s.name_order_store.borrow();
            (
                store.get_need_to_be_check_name_availability_principals(now),
                store.get_expired_quota_order_user_principals(now),
            )
        });

        // log count
        info!(
            "need_check_orders: {}, expired_order: {}",
            need_check_orders.len(),
            expired_order.len()
        );

        // cancel expired orders
        for user in expired_order {
            self.cancel_order(&user, now)?;
        }

        let mut need_to_be_cancel_users = vec![];
        // check orders
        STATE.with(|s| {
            let store = s.registration_store.borrow();
            let name_order_store = s.name_order_store.borrow();
            for user in need_check_orders.iter() {
                let user_order = name_order_store.get_order(user).unwrap();
                let name = user_order.name();
                if store.registrations.contains_key(name) {
                    need_to_be_cancel_users.push(user);
                }
            }
        });

        info!("need_to_be_cancel_users: {}", need_to_be_cancel_users.len());
        for user in need_to_be_cancel_users {
            self.cancel_order(user, now)?;
        }
        Ok(true)
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

    pub async fn transfer_from(&self, caller: &Principal, name: &str) -> ServiceResult<bool> {
        let name = validate_name(name)?;
        must_not_anonymous(caller)?;
        STATE.with(|s| {
            let store = s.registration_approval_store.borrow_mut();
            if !store.is_approved_to(&name, caller) {
                return Err(NamingError::PermissionDenied);
            }

            Ok(())
        })?;

        self.transfer_core(&name, caller).await
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

        if let Err(s) = STATE.with(|s| {
            let mut name_order_store = s.name_order_store.borrow_mut();
            name_order_store.add_handling_name(&first_level_name)
        }) {
            error!("error adding handling name: {}", s);
            return Err(NamingError::Conflict);
        };

        let result = self
            .token_service
            .transfer_from(
                caller.to_text().as_str(),
                DICP_RECEIVER.deref(),
                Nat::from(request.approve_amount),
                now,
            )
            .await;
        STATE.with(|s| {
            let mut name_order_store = s.name_order_store.borrow_mut();
            name_order_store
                .remove_handling_name(&first_level_name)
                .unwrap();
        });
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
}

fn ensure_no_pending_name_order(caller: &Principal) -> ServiceResult<()> {
    STATE.with(|state| {
        let store = state.name_order_store.borrow();
        if store.get_order(caller).is_some() {
            return Err(NamingError::PendingOrder);
        };
        Ok(())
    })
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

impl From<&NameOrder> for RegisterCoreContext {
    fn from(order: &NameOrder) -> Self {
        RegisterCoreContext::new(
            order.name().to_string(),
            AuthPrincipal(order.created_user()),
            order.years(),
            TimeInNs(order.created_at()),
            false,
        )
    }
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

#[cfg(test)]
mod tests;

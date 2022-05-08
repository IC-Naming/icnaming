use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk::api;
use log::{debug, error, info, trace, warn};
use num_bigint::BigUint;
use num_traits::ToPrimitive;

use common::canister_api::ic_impl::{CyclesMintingApi, ICNamingLedgerApi, RegistryApi};
use common::canister_api::{ICyclesMintingApi, IICNamingLedgerApi, IRegistryApi};
use common::constants::*;
use common::dto::{GetPageInput, GetPageOutput, ImportQuotaRequest, ImportQuotaStatus};
use common::errors::ICNSError::RemoteError;
use common::errors::{ICNSError, ICNSResult};
use common::ic_api::wrapper::ICStaticApi;
use common::ic_api::IClock;
use common::icnaming_ledger_types::{
    AddPaymentRequest, BlockHeight, ICPTs, PaymentId, RefundPaymentRequest, RefundPaymentResponse,
    SyncICPPaymentRequest, VerifyPaymentResponse,
};
use common::metrics_encoder::MetricsEncoder;
use common::named_canister_ids::{
    get_named_get_canister_id, CANISTER_NAME_REGISTRAR_CONTROL_GATEWAY, CANISTER_NAME_RESOLVER,
};
use common::named_principals::{NAME_DPRINCIPALS, PRINCIPAL_NAME_TIMER_TRIGGER};
use common::naming::{normalize_name, NameParseResult};
use common::permissions::{is_admin, must_be_named_canister, must_be_system_owner};
use common::permissions::{must_be_named_principal, must_not_anonymous};

use crate::name_locker::{try_lock_name, unlock_name};
use crate::name_order_store::{GetNameOrderResponse, NameOrder, NameOrderStatus};
use crate::quota_order_store::{
    GetOrderOutput, ICPMemo, PaymentMemo, PaymentType, PlaceOrderOutput, QuotaOrderDetails,
    QuotaOrderPayment, QuotaOrderStatus,
};
use crate::registration_store::{Registration, RegistrationDetails, RegistrationDto};
use crate::reserved_list::RESERVED_NAMES;
use crate::state::*;
use crate::user_quota_store::QuotaType;

#[derive(Deserialize, CandidType)]
pub struct SubmitOrderRequest {
    pub name: String,
    pub years: u32,
}

#[derive(Deserialize, CandidType)]
pub struct SubmitOrderResponse {
    pub order: GetNameOrderResponse,
}

/// Check if name is available.
/// Returns true if name is available.
/// * `name` - name to check, e.g. "hello.icp"
pub struct RegistrarService {
    pub registry_api: Arc<dyn IRegistryApi>,
    pub clock: Arc<dyn IClock>,
    pub icnaming_ledger_api: Arc<dyn IICNamingLedgerApi>,
    pub cycles_minting_api: Arc<dyn ICyclesMintingApi>,
}

impl RegistrarService {}

impl Debug for RegistrarService {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, stringify!(RegistrarService))
    }
}

impl Default for RegistrarService {
    fn default() -> Self {
        RegistrarService::new()
    }
}

impl RegistrarService {
    pub fn new() -> RegistrarService {
        RegistrarService {
            registry_api: Arc::new(RegistryApi),
            clock: Arc::new(ICStaticApi::new()),
            icnaming_ledger_api: Arc::new(ICNamingLedgerApi::new()),
            cycles_minting_api: Arc::new(CyclesMintingApi::new()),
        }
    }

    pub(crate) fn get_stats(&self, _now: u64) -> Stats {
        let mut stats = Stats::default();
        stats.cycles_balance = api::canister_balance();
        NAME_LOCKER.with(|name_locker| {
            stats.name_lock_count = name_locker.borrow().get_count() as u64;
        });
        STATE.with(|s| {
            {
                let store = s.registration_store.borrow();
                // count distinct owners of registered names
                let mut owners = HashSet::new();
                for registration in store.get_registrations().values() {
                    owners.insert(registration.get_owner());
                }
                stats.user_count = owners.len() as u64;
            }
            {
                let mut user_quota_count = HashMap::new();
                let quota_types = vec![
                    QuotaType::LenGte(1),
                    QuotaType::LenGte(2),
                    QuotaType::LenGte(3),
                    QuotaType::LenGte(4),
                    QuotaType::LenGte(5),
                    QuotaType::LenGte(6),
                    QuotaType::LenGte(7),
                    QuotaType::LenEq(1),
                    QuotaType::LenEq(2),
                    QuotaType::LenEq(3),
                    QuotaType::LenEq(4),
                    QuotaType::LenEq(5),
                    QuotaType::LenEq(6),
                    QuotaType::LenEq(7),
                ];
                for quota_type in quota_types {
                    user_quota_count.insert(quota_type, 0u64);
                }
                let store = s.user_quota_store.borrow();
                let quotas = store.get_user_quotas();
                for user_quota in quotas.values() {
                    for (t, user_count) in user_quota {
                        let count = user_quota_count.entry(*t).or_insert(0);
                        *count += *user_count as u64;
                    }
                }
                let mut user_quota_count_stats = HashMap::new();
                for (quota_type, count) in user_quota_count {
                    let type_str = quota_type.to_string().replace('(', "").replace(')', "");
                    user_quota_count_stats.entry(type_str).or_insert(count);
                }
                stats.user_quota_count = user_quota_count_stats;
            }
            {
                let manager = s.quota_order_store.borrow();
                let orders = manager.get_all_orders();
                let mut count_by_status = HashMap::new();
                count_by_status.insert(format!("{:?}", QuotaOrderStatus::New).to_lowercase(), 0);
                count_by_status.insert(format!("{:?}", QuotaOrderStatus::Done).to_lowercase(), 0);
                count_by_status.insert(
                    format!("{:?}", QuotaOrderStatus::Canceled).to_lowercase(),
                    0,
                );

                for (_, order) in orders {
                    let order = order.borrow();
                    let status: &QuotaOrderStatus = order.status();
                    let count = count_by_status
                        .entry(format!("{:?}", status).to_lowercase())
                        .or_insert(0);
                    *count += 1;
                }

                stats.user_quota_order_count = count_by_status;
            }
            {
                let store = s.name_order_store.borrow();
                let orders = store.get_all_orders();
                let mut count_by_status = HashMap::new();
                count_by_status.insert(format!("{:?}", NameOrderStatus::New).to_lowercase(), 0);
                count_by_status.insert(format!("{:?}", NameOrderStatus::Done).to_lowercase(), 0);
                count_by_status.insert(
                    format!("{:?}", NameOrderStatus::WaitingToRefund).to_lowercase(),
                    0,
                );
                count_by_status
                    .insert(format!("{:?}", NameOrderStatus::Canceled).to_lowercase(), 0);

                for (_, order) in orders {
                    let status: &NameOrderStatus = order.order_status();
                    let count = count_by_status
                        .entry(format!("{:?}", status).to_lowercase())
                        .or_insert(0);
                    *count += 1;
                }

                stats.user_name_order_count_by_status = count_by_status;
            }
            {
                let store = s.registration_store.borrow();
                let count = store.get_registrations().len();

                stats.registration_count = count as u64;
            }
            {
                let store = s.payment_store.borrow();
                stats.payment_version = store.get_payment_version_synced_up_to().unwrap_or(0);
            }
        });
        MERTRICS_COUNTER.with(|c| {
            let counter = c.borrow();
            stats.last_xdr_permyriad_per_icp = counter.last_xdr_permyriad_per_icp;
            stats.last_timestamp_seconds_xdr_permyriad_per_icp =
                counter.last_timestamp_seconds_xdr_permyriad_per_icp;
            stats.name_order_placed_count = counter.name_order_placed_count;
            stats.name_order_paid_count = counter.name_order_paid_count;
            stats.name_order_cancelled_count = counter.name_order_cancelled_count;
            stats.new_registered_name_count = counter.new_registered_name_count;
        });

        stats
    }

    pub(crate) fn get_names(
        &self,
        owner: &Principal,
        input: &GetPageInput,
    ) -> ICNSResult<GetPageOutput<RegistrationDto>> {
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

    pub(crate) fn get_details(&self, name: &str) -> ICNSResult<RegistrationDetails> {
        let name = normalize_name(name);
        STATE.with(|s| {
            let store = s.registration_store.borrow();
            let registrations = store.get_registrations();
            let registration = registrations.get(&name);
            if registration.is_none() {
                return Err(ICNSError::RegistrationNotFound);
            }
            Ok(RegistrationDetails::from(registration.unwrap()))
        })
    }

    pub(crate) fn get_all_details(
        &self,
        caller: &Principal,
        input: &GetPageInput,
    ) -> ICNSResult<Vec<RegistrationDetails>> {
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

    pub(crate) fn get_owner(&self, name: &str) -> ICNSResult<Principal> {
        let name = normalize_name(name);
        STATE.with(|s| {
            let store = s.registration_store.borrow();
            let registrations = store.get_registrations();
            let registration = registrations.get(&name);
            if registration.is_none() {
                return Err(ICNSError::RegistrationNotFound);
            }
            Ok(registration.unwrap().get_owner().to_owned())
        })
    }

    pub(crate) fn get_name_expires(&self, name: &str) -> ICNSResult<u64> {
        let name = self.normalize_name(&name);
        STATE.with(|s| {
            let store = s.registration_store.borrow();
            let registration = store.get_registrations().get(&name);
            if let Some(registration) = registration {
                return Ok(registration.get_expired_at() / 1_000_000);
            }
            Err(ICNSError::RegistrationNotFound)
        })
    }
    pub fn normalize_name(&self, name: &str) -> String {
        normalize_name(name)
    }

    pub fn validate_quota(
        &self,
        name: &NameParseResult,
        owner: &Principal,
        quota_type: &QuotaType,
        quota_required: u32,
    ) -> Result<(), String> {
        let first = name.get_current_level().unwrap();
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
        let result = STATE.with(|s| {
            let user_quota_manager = s.user_quota_store.borrow();
            let quota = user_quota_manager
                .get_quota(owner, &quota_type)
                .unwrap_or(0);
            if quota < quota_required {
                return Err(format!("User has no quota for {}", quota_type));
            }
            return Ok(());
        });
        result
    }

    fn validate_name(&self, name: &str) -> ICNSResult<NameParseResult> {
        assert!(name.len() > 0);
        let result = NameParseResult::parse(name);
        if result.get_level_count() != 2 {
            return Err(ICNSError::InvalidName {
                reason: "it must be second level name".to_string(),
            });
        }
        if result.get_top_level().unwrap() != TOP_LABEL {
            return Err(ICNSError::InvalidName {
                reason: format!("top level of name must be {}", TOP_LABEL),
            });
        }
        let first = result.get_current_level().unwrap();
        if first.len() > 63 {
            return Err(ICNSError::InvalidName {
                reason: "second level name must be less than 64 characters".to_string(),
            });
        }

        if !first.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(ICNSError::InvalidName {
                reason: "name must be alphanumeric or -".to_string(),
            });
        }
        return Ok(result);
    }

    fn validate_year(&self, years: u32) -> ICNSResult<()> {
        STATE.with(|s| {
            let settings = s.settings.borrow();
            if !settings.is_year_valid(years) {
                return Err(ICNSError::YearsRangeError {
                    min: settings.min_year,
                    max: settings.max_year,
                });
            }
            Ok(())
        })
    }

    pub async fn register_from_gateway(
        &mut self,
        caller: &Principal,
        name: &str,
        owner: &Principal,
        now: u64,
    ) -> ICNSResult<bool> {
        must_be_named_canister(caller, CANISTER_NAME_REGISTRAR_CONTROL_GATEWAY)?;
        let years = 1;
        let admin_import = true;
        let quota_owner = &get_named_get_canister_id(CANISTER_NAME_REGISTRAR_CONTROL_GATEWAY);
        let quota_type = QuotaType::LenGte(1);
        self.register(
            name,
            owner,
            years,
            now,
            quota_owner,
            quota_type,
            admin_import,
        )
        .await
    }

    pub async fn register(
        &mut self,
        name: &str,
        owner: &Principal,
        years: u32,
        now: u64,
        quota_owner: &Principal,
        quota_type: QuotaType,
        admin_import: bool,
    ) -> ICNSResult<bool> {
        let name = self.normalize_name(&name);

        // validate name
        let name_result = self.validate_name(&name)?;

        // validate user
        must_not_anonymous(owner)?;

        // validate quota_owner
        must_not_anonymous(quota_owner)?;

        // validate year
        self.validate_year(years)?;

        // validate quota
        let quota_result = self.validate_quota(&name_result, quota_owner, &quota_type, years);
        if quota_result.is_err() {
            return Err(ICNSError::InvalidName {
                reason: quota_result.err().unwrap().to_string(),
            });
        }

        // check reservation if not admin import
        if !admin_import {
            // check reserved names
            if RESERVED_NAMES.contains(&name_result.get_current_level().unwrap().as_str()) {
                return Err(ICNSError::RegistrationHasBeenTaken);
            }
        }

        STATE.with(|s| {
            let store = s.registration_store.borrow();
            let registrations = store.get_registrations();
            if registrations.contains_key(&name) {
                return Err(ICNSError::RegistrationHasBeenTaken);
            }
            Ok(())
        })?;

        // update quota before await in case of concurrent register
        STATE.with(|s| {
            let mut user_quota_manager = s.user_quota_store.borrow_mut();
            let result = user_quota_manager.sub_quota(quota_owner, &quota_type, years);
            assert_eq!(result, true);
        });

        // TODO adjusts to date format w/o seconds
        // keep date for now_in_ns
        let expired_at = now + year_to_ns(years);
        let resolver = get_named_get_canister_id(CANISTER_NAME_RESOLVER);
        let registration = Registration::new(owner.clone(), name.clone(), expired_at, now);
        trace!("registering {:?}", registration);
        let api_result = self
            .registry_api
            .set_subdomain_owner(
                name_result.get_current_level().unwrap().clone(),
                TOP_LABEL.to_string(),
                owner.clone(),
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
            // rollback quota
            STATE.with(|s| {
                let mut user_quota_manager = s.user_quota_store.borrow_mut();
                user_quota_manager.add_quota(quota_owner.clone(), quota_type, years);
            });
            Err(RemoteError(api_result.err().unwrap()))
        }
    }

    pub fn available(&self, name: &str) -> ICNSResult<NameParseResult> {
        let name = self.normalize_name(name);
        let result = self.validate_name(&name)?;

        // check reserved names
        if RESERVED_NAMES.contains(&result.get_current_level().unwrap().as_str()) {
            return Err(ICNSError::RegistrationHasBeenTaken);
        }

        STATE.with(|s| {
            let store = s.registration_store.borrow();
            let registrations = store.get_registrations();
            if registrations.contains_key(&name) {
                return Err(ICNSError::RegistrationHasBeenTaken);
            }
            Ok(result)
        })
    }

    pub fn clean_expired(&mut self, _now_in_ms: u64) -> ICNSResult<()> {
        todo!("clean up")
    }

    pub fn has_pending_order(&self, caller: &Principal) -> ICNSResult<bool> {
        must_not_anonymous(caller)?;
        Ok(STATE.with(|s| {
            let name_order_store = s.name_order_store.borrow();
            name_order_store.has_name_order(caller)
        }))
    }

    pub fn get_pending_order(
        &self,
        caller: &Principal,
    ) -> ICNSResult<Option<GetNameOrderResponse>> {
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
    ) -> ICNSResult<SubmitOrderResponse> {
        // check
        must_not_anonymous(caller)?;
        ensure_no_pending_name_order(caller)?;
        ensure_no_pending_quota_order(caller)?;
        let name_result = self.available(request.name.as_str())?;
        self.validate_year(request.years)?;
        let current_level = name_result.get_current_level().unwrap();
        let name_length = current_level.chars().count() as u8;
        let length_limit = 6;
        if name_length < length_limit {
            return Err(ICNSError::InvalidName {
                reason: format!(
                    "the name need to be at least {} characters long",
                    length_limit,
                ),
            });
        }

        // place quota order
        let quota_type_len = min(name_length, MAX_LENGTH_OF_NAME_QUOTA_TYPE);
        let quote_type = QuotaType::LenGte(quota_type_len);
        let mut quotas = HashMap::new();
        quotas.insert(quote_type.clone(), request.years);
        let mut details = HashMap::new();
        details.insert(caller.clone(), quotas);
        self.place_quota_order(caller, now, details).await?;

        ensure_no_pending_name_order(caller)?;
        let (payment_memo, payment_account_id, amount, payment_id) = STATE.with(|s| {
            let quota_order_manager = s.quota_order_store.borrow_mut();
            let order = quota_order_manager.get_order(caller).unwrap();
            let order = order.borrow();
            let payment = order.payment();
            (
                payment.payment_memo().clone(),
                payment.payment_account_id().clone(),
                payment.amount().clone(),
                payment.payment_id().clone(),
            )
        });

        // place name order
        let get_order_result: GetNameOrderResponse = STATE.with(|s| {
            let mut name_order_store = s.name_order_store.borrow_mut();
            name_order_store.add_name_order(
                caller,
                request.name.as_str(),
                request.years,
                &payment_memo,
                amount.clone(),
                &payment_id,
                &payment_account_id,
                &quote_type,
            );
            name_order_store.get_order(caller).unwrap().into()
        });

        Ok(SubmitOrderResponse {
            order: get_order_result,
        })
    }

    pub fn cancel_order(&self, caller: &Principal, now: u64) -> ICNSResult<bool> {
        must_not_anonymous(caller)?;
        self.cancel_quota_order(caller, now)?;
        STATE.with(|s| {
            let mut name_order_store = s.name_order_store.borrow_mut();
            let order = name_order_store.get_order(caller);
            if order.is_none() {
                return Err(ICNSError::OrderNotFound);
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

    pub async fn refund_order(&self, caller: &Principal, now: u64) -> ICNSResult<bool> {
        must_not_anonymous(caller)?;
        let payment_id: PaymentId = STATE.with(|s| {
            let name_order_store = s.name_order_store.borrow_mut();
            let order = name_order_store.get_order(caller);

            if order.is_none() {
                Err(ICNSError::OrderNotFound)
            } else {
                let order = order.unwrap();
                let status = order.order_status();
                assert_eq!(status, &NameOrderStatus::WaitingToRefund);
                Ok(order.payment_id().clone())
            }
        })?;

        let refund_payment_response = self
            .icnaming_ledger_api
            .refund_payment(RefundPaymentRequest {
                payment_id: payment_id.clone(),
            })
            .await?;

        match refund_payment_response {
            RefundPaymentResponse::Refunded { .. } => {
                self.cancel_quota_order(caller, now)?;
                STATE.with(|s| {
                    let mut name_order_store = s.name_order_store.borrow_mut();
                    let order = name_order_store.get_order(caller).unwrap();
                    let status = order.order_status();
                    assert_eq!(status, &NameOrderStatus::WaitingToRefund);
                    debug!("refunded payment for order {}", order.created_user());
                    name_order_store.remove_name_order(caller);
                });
                Ok(true)
            }
            RefundPaymentResponse::Refunding => Err(ICNSError::RefundFailed),
            RefundPaymentResponse::RefundFailed => Err(ICNSError::RefundFailed),
            RefundPaymentResponse::PaymentNotFound => Err(ICNSError::OrderNotFound),
        }
    }

    pub async fn confirm_pay_order(
        &mut self,
        now: u64,
        caller: &Principal,
        block_height: BlockHeight,
    ) -> ICNSResult<bool> {
        self.has_pending_order(caller)?;

        let sync_result = self
            .icnaming_ledger_api
            .sync_icp_payment(SyncICPPaymentRequest { block_height })
            .await?;

        if let Some(payment_id) = sync_result.payment_id {
            let verify_result = sync_result.verify_payment_response.unwrap();
            match verify_result {
                VerifyPaymentResponse::NeedMore { .. } => {
                    trace!("Need more payment data for payment id {}", payment_id);
                    Ok(false)
                }
                VerifyPaymentResponse::Paid { .. } => {
                    info!("Payment {} paid", payment_id);
                    let result = self.apply_paid_order(payment_id, now).await;
                    Ok(result)
                }
                VerifyPaymentResponse::PaymentNotFound => {
                    todo!("Payment not found, clean order");
                    Ok(false)
                }
            }
        } else {
            Ok(false)
        }
    }

    pub async fn apply_paid_order(&mut self, payment_id: PaymentId, now_in_ns: u64) -> bool {
        enum NameOrderHandlingStatus {
            NeedToHandle(NameOrder),
            AlreadyHandled,
            Conflicted(NameOrder),
        }
        let name_order_handling_status = STATE.with(|s| {
            let mut name_order_store = s.name_order_store.borrow_mut();
            let order = name_order_store.get_order_by_payment_id(&payment_id);
            if order.is_none() {
                debug!(
                    "order not found for payment id {}, it should be handle before",
                    payment_id
                );
                return NameOrderHandlingStatus::AlreadyHandled;
            }

            let order = order.unwrap();
            let order = order.clone();

            // lock payment id
            let handling_result = name_order_store.add_handling_name(order.name().clone().as_str());
            if handling_result.is_err() {
                error!("failed to add handling name {}", order.name());
                return NameOrderHandlingStatus::Conflicted(order);
            }

            return NameOrderHandlingStatus::NeedToHandle(order);
        });
        let name_claimed = match name_order_handling_status {
            NameOrderHandlingStatus::NeedToHandle(order) => {
                let user: &Principal = order.created_user();
                let name: &str = order.name().as_str();
                assert!(order.order_status() == &NameOrderStatus::New);
                // if name is not available or it is claiming by other user, go to refund it
                if self.available(name).is_err() {
                    // go to refund
                    STATE.with(|s| {
                        let mut name_order_store = s.name_order_store.borrow_mut();
                        name_order_store.waiting_to_refund(user);
                    });
                    return false;
                } else {
                    if self.exits_by_payment_id(payment_id) {
                        // complete quota order to claim name
                        self.paid_quota_order(payment_id, now_in_ns);
                    } else {
                        warn!(
                    "there is no quota order found with payment_id:{}, it is not a common case but leave it pass and try to claim name",
                    name
                );
                    }

                    // complete name order
                    let result = self
                        .register(
                            name,
                            user,
                            order.years().clone(),
                            now_in_ns,
                            order.created_user(),
                            order.quota_type().clone(),
                            false,
                        )
                        .await;
                    if !(result.is_ok()) {
                        warn!("failed to register name {}", name);
                    }
                    // always remove name order, if name is not registered, take name, if name is not available, just remove name order and leave quota to user.
                    // release payment id
                    STATE.with(|s| {
                        let mut store = s.name_order_store.borrow_mut();
                        store.remove_name_order(user);
                        store.remove_handling_name(name).unwrap();
                    });

                    return result.is_ok();
                }
            }
            NameOrderHandlingStatus::AlreadyHandled => true,
            NameOrderHandlingStatus::Conflicted(order) => {
                let user: &Principal = order.created_user();

                // go to refund
                STATE.with(|s| {
                    let mut name_order_store = s.name_order_store.borrow_mut();
                    name_order_store.waiting_to_refund(user);
                });
                false
            }
        };

        return name_claimed;
    }

    pub fn add_quota(
        &mut self,
        caller: &Principal,
        quota_owner: Principal,
        quota_type: QuotaType,
        diff: u32,
    ) -> ICNSResult<bool> {
        must_be_system_owner(caller)?;
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
    ) -> ICNSResult<bool> {
        must_be_system_owner(caller)?;
        STATE.with(|s| {
            let mut user_quota_manager = s.user_quota_store.borrow_mut();
            user_quota_manager.sub_quota(&quota_owner, &quota_type, diff);
        });
        Ok(true)
    }

    pub fn get_quota(
        &self,
        caller: &Principal,
        quota_owner: Principal,
        quota_type: QuotaType,
    ) -> ICNSResult<u32> {
        must_not_anonymous(caller)?;
        STATE.with(|s| {
            let user_quota_manager = s.user_quota_store.borrow();
            let target_user = if is_admin(caller) {
                &quota_owner
            } else {
                caller
            };
            Ok(user_quota_manager
                .get_quota(target_user, &quota_type)
                .unwrap_or(0))
        })
    }

    fn get_quota_order(&self, caller: &Principal) -> ICNSResult<Option<GetOrderOutput>> {
        must_not_anonymous(caller)?;
        STATE.with(|s| {
            let manager = s.quota_order_store.borrow();
            let order_ref = manager.get_order(caller);
            if order_ref.is_none() {
                Ok(None)
            } else {
                let order_ref = order_ref.unwrap();
                let order = order_ref.borrow();
                let order = order.deref().into();
                Ok(Some(order))
            }
        })
    }

    async fn place_quota_order(
        &self,
        caller: &Principal,
        now: u64,
        details: QuotaOrderDetails,
    ) -> ICNSResult<PlaceOrderOutput> {
        must_not_anonymous(caller)?;
        validate_quota_order_details(&details)?;
        ensure_no_pending_quota_order(caller)?;

        let response = self
            .cycles_minting_api
            .get_icp_xdr_conversion_rate()
            .await?;
        let icp_xdr_conversion_rate = response.data.xdr_permyriad_per_icp;
        assert!(icp_xdr_conversion_rate > 0);

        let price_icp_e8s =
            get_price_for_quota_order_details_in_icp_e8s(&details, icp_xdr_conversion_rate);
        assert!(price_icp_e8s > 0u64);
        assert!(price_icp_e8s <= u64::MAX);
        debug!("place_order: price_icp_e8s={}", price_icp_e8s.to_string());

        let add_payment_response = self
            .icnaming_ledger_api
            .add_payment(AddPaymentRequest {
                amount: ICPTs::new(price_icp_e8s.to_u64().unwrap()),
                created_remark: "".to_string(),
            })
            .await?;

        ensure_no_pending_quota_order(caller)?;

        let memo = add_payment_response.memo.0;
        let payment = QuotaOrderPayment::new(
            add_payment_response.payment_id,
            PaymentType::ICP,
            Nat(BigUint::from(price_icp_e8s)),
            PaymentMemo::ICP(ICPMemo(memo)),
            add_payment_response.payment_account_id,
        );

        STATE.with(|s| {
            let mut manager = s.quota_order_store.borrow_mut();
            manager.add_order(caller.to_owned(), details, now, payment);
        });

        let order = self.get_quota_order(caller).unwrap().unwrap();
        MERTRICS_COUNTER.with(|c| {
            let mut counter = c.borrow_mut();
            counter.last_xdr_permyriad_per_icp = icp_xdr_conversion_rate;
            counter.last_timestamp_seconds_xdr_permyriad_per_icp = now;
        });
        Ok(PlaceOrderOutput { order })
    }

    fn cancel_quota_order(&self, caller: &Principal, now: u64) -> ICNSResult<bool> {
        must_not_anonymous(caller)?;
        STATE.with(|s| {
            let manager = s.quota_order_store.borrow();
            let order_ref = manager.get_order(caller);
            if order_ref.is_none() {
                return Err(ICNSError::OrderNotFound);
            };
            let mut order = order_ref.unwrap().borrow_mut();
            order.cancel(now);
            Ok(true)
        })?;

        // remove order
        STATE.with(|s| {
            let mut manager = s.quota_order_store.borrow_mut();
            manager.remove_order_by_principal(caller);

            // TODO: add logging
            Ok(true)
        })
    }

    fn paid_quota_order(&self, payment_id: PaymentId, now: u64) {
        // update order
        let user: Principal = STATE.with(|s| {
            let manager = s.quota_order_store.borrow();
            let order_ref = manager.get_order_by_payment_id(&payment_id);
            assert!(order_ref.is_some());
            let order_ref = order_ref.unwrap();
            let mut order = order_ref.borrow_mut();
            assert_eq!(order.is_paid(), false);
            order.verified_payment(now);
            let details: &QuotaOrderDetails = order.details();
            apply_quota_order_details(details);
            order.created_user().clone()
        });

        STATE.with(|s| {
            let mut manager = s.quota_order_store.borrow_mut();
            manager.remove_order_by_principal(&user);
        });
    }

    fn exits_by_payment_id(&self, payment_id: PaymentId) -> bool {
        STATE.with(|s| {
            let manager = s.quota_order_store.borrow();
            let order_ref = manager.get_order_by_payment_id(&payment_id);
            order_ref.is_some()
        })
    }

    pub async fn get_price_table(&self) -> ICNSResult<PriceTable> {
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
    ) -> ICNSResult<ImportQuotaStatus> {
        must_be_named_canister(caller, CANISTER_NAME_REGISTRAR_CONTROL_GATEWAY)?;
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
                store.add_quota(
                    item.owner,
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

    pub fn cancel_expired_orders(&self, now: u64) -> ICNSResult<bool> {
        let (need_check_orders, expired_order) = STATE.with(|s| {
            let store = s.quota_order_store.borrow();
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
                let name: &String = user_order.name();
                if store.registrations.contains_key(name.as_str()) {
                    need_to_be_cancel_users.push(user);
                }
            }
        });

        info!("need_to_be_cancel_users: {}", need_to_be_cancel_users.len());
        for user in need_to_be_cancel_users {
            self.cancel_order(&user, now)?;
        }
        Ok(true)
    }

    fn is_name_owner(&self, name: &str, caller: &Principal) -> ICNSResult<Principal> {
        STATE.with(|s| {
            let store = s.registration_store.borrow();
            let registrations = store.get_registrations();
            let registration = registrations.get(name);
            if registration.is_none() {
                return Err(ICNSError::RegistrationNotFound);
            }
            let registration = registration.unwrap();
            let owner = registration.get_owner();

            if !owner.eq(caller) {
                return Err(ICNSError::PermissionDenied);
            }

            Ok(owner)
        })
    }

    async fn reclaim_name(&self, name: &str, caller: &Principal) -> ICNSResult<bool> {
        self.validate_name(name)?;
        must_not_anonymous(caller)?;
        let registration_owner = self.is_name_owner(name, caller)?;
        debug!("reclaim name: {} to user {}", name, &registration_owner);

        let resolver = get_named_get_canister_id(CANISTER_NAME_RESOLVER);
        try_lock_name(name)?;
        let reclaim_result = self
            .registry_api
            .reclaim_name(name.to_string(), registration_owner.clone(), resolver)
            .await;
        unlock_name(name);

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
                Err(RemoteError(e).into())
            }
        };
        result
    }

    async fn transfer_core(&self, name: &str, new_owner: &Principal) -> ICNSResult<bool> {
        STATE.with(|s| {
            let store = s.registration_store.borrow();
            if !store.has_registration(name) {
                return Err(ICNSError::RegistrationNotFound);
            }
            Ok(())
        })?;
        try_lock_name(name)?;
        let registry_result = self
            .registry_api
            .transfer(
                name.to_string(),
                new_owner.clone(),
                get_named_get_canister_id(CANISTER_NAME_RESOLVER),
            )
            .await;
        unlock_name(name);
        registry_result?;

        STATE.with(|s| {
            let mut store = s.registration_store.borrow_mut();
            store.transfer_registration(name.to_string(), new_owner.clone());

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
    ) -> ICNSResult<bool> {
        self.validate_name(name)?;
        must_not_anonymous(caller)?;
        must_not_anonymous(&new_owner)?;
        let _ = self.is_name_owner(name, caller)?;
        assert_ne!(caller, &new_owner);

        self.transfer_core(name, &new_owner).await
    }

    // TODO: remove this function when all assignment is done
    pub async fn transfer_by_admin(
        &self,
        name: &str,
        caller: &Principal,
        new_owner: Principal,
    ) -> ICNSResult<bool> {
        must_be_system_owner(caller)?;
        let name_parse_result = self.validate_name(name)?;
        assert!(RESERVED_NAMES
            .iter()
            .find(|n| *n == name_parse_result.get_current_level().unwrap())
            .is_some());
        must_not_anonymous(&new_owner)?;

        self.transfer_core(name, &new_owner).await
    }

    pub fn approve(
        &self,
        caller: &Principal,
        now: u64,
        name: &str,
        to: Principal,
    ) -> ICNSResult<bool> {
        self.validate_name(name)?;
        must_not_anonymous(caller)?;
        let _ = self.is_name_owner(name, caller)?;
        assert_ne!(caller, &to);

        STATE.with(|s| {
            let mut store = s.registration_approval_store.borrow_mut();
            store.set_approval(name, &to, now);
            Ok(true)
        })
    }

    pub async fn transfer_from(&self, caller: &Principal, name: &str) -> ICNSResult<bool> {
        self.validate_name(name)?;
        must_not_anonymous(caller)?;
        STATE.with(|s| {
            let store = s.registration_approval_store.borrow_mut();
            if !store.is_approved_to(name, caller) {
                return Err(ICNSError::PermissionDenied);
            }

            Ok(())
        })?;

        self.transfer_core(name, caller).await
    }

    pub fn transfer_from_quota(
        &self,
        caller: &Principal,
        from: Principal,
        to: Principal,
        quota_type: QuotaType,
        diff: u32,
    ) -> ICNSResult<bool> {
        // TODO: change validation caller to named canister
        must_be_system_owner(caller)?;
        must_not_anonymous(&to)?;
        must_not_anonymous(&from)?;
        assert!(diff > 0);

        STATE.with(|s| {
            let mut store = s.user_quota_store.borrow_mut();
            let quota_count = store.get_quota(&from, &quota_type).unwrap_or(0);
            if quota_count < diff {
                return Err(ICNSError::InsufficientQuota);
            }

            store.sub_quota(&from, &quota_type, diff);
            store.add_quota(to.clone(), quota_type.clone(), diff);
            info!(
                "transfer quota: {} from user {} to user {}, diff: {}",
                quota_type, &from, &to, diff
            );
            Ok(true)
        })
    }

    pub fn transfer_quota(
        &self,
        caller: &Principal,
        to: &Principal,
        quota_type: QuotaType,
        diff: u32,
    ) -> ICNSResult<bool> {
        must_not_anonymous(caller)?;
        must_not_anonymous(to)?;
        assert_ne!(caller, to);

        STATE.with(|s| {
            let mut store = s.user_quota_store.borrow_mut();
            Ok(store.transfer_quota(caller, to, &quota_type, diff))
        })
    }

    pub fn unlock_names(&self, caller: &Principal, names: Vec<&str>) -> ICNSResult<bool> {
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
    ) -> ICNSResult<Vec<RegistrationDetails>> {
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

    pub fn set_maintaining_time(&self, caller: &Principal, time: u64) -> ICNSResult<bool> {
        must_be_system_owner(caller)?;
        STATE.with(|s| {
            let mut settings = s.settings.borrow_mut();
            settings.set_maintaining_time(time);
        });
        Ok(true)
    }
}

fn apply_quota_order_details(details: &QuotaOrderDetails) {
    STATE.with(|s| {
        let mut user_quota_manager = s.user_quota_store.borrow_mut();
        for (user, quotas) in details.iter() {
            for (quota_type, quota) in quotas.iter() {
                user_quota_manager.add_quota(user.clone(), quota_type.clone(), quota.clone());
            }
        }
    })
}

fn year_to_ns(years: u32) -> u64 {
    years as u64 * 365 * 24 * 60 * 60 * 1000 * 1_000_000
}

fn validate_quota_order_details(details: &QuotaOrderDetails) -> ICNSResult<()> {
    if details.len() == 0 {
        return Err(ICNSError::InvalidQuotaOrderDetails);
    }

    // validate each item
    for (user, quotas) in details.iter() {
        must_not_anonymous(user)?;
        for (_, amount) in quotas.iter() {
            if amount == &0 {
                return Err(ICNSError::InvalidQuotaOrderDetails);
            }
            if amount > &MAX_QUOTA_ORDER_AMOUNT_EACH_TYPE {
                return Err(ICNSError::InvalidQuotaOrderDetails);
            }
        }
    }

    return Ok(());
}

fn get_price_for_quota_order_details_in_icp_e8s(
    details: &QuotaOrderDetails,
    xdr_permyriad_per_icp: u64,
) -> u64 {
    #[cfg(any(feature = "dev_canister", feature = "production_canister"))]
    {
        let mut result = 0u64;
        for (_, quotas) in details.iter() {
            for (quota_type, amount) in quotas.iter() {
                let price = get_quota_type_price_in_icp_e8s(quota_type, xdr_permyriad_per_icp)
                    * amount.clone() as u64;
                result = result + price;
            }
        }
        result
    }
    #[cfg(feature = "staging_canister")]
    {
        20_000u64
    }
}

fn get_quota_type_price_in_icp_e8s(quota_type: &QuotaType, xdr_permyriad_per_icp: u64) -> u64 {
    match quota_type {
        QuotaType::LenEq(len) => get_price_in_icp_e8s(len.clone(), xdr_permyriad_per_icp),
        QuotaType::LenGte(len) => get_price_in_icp_e8s(len.clone(), xdr_permyriad_per_icp),
    }
}

fn ensure_no_pending_quota_order(caller: &Principal) -> ICNSResult<()> {
    STATE.with(|s| {
        let manager = s.quota_order_store.borrow_mut();
        if manager.has_pending_order(caller) {
            return Err(ICNSError::PendingOrder);
        };
        Ok(())
    })
}

fn ensure_no_pending_name_order(caller: &Principal) -> ICNSResult<()> {
    STATE.with(|state| {
        let store = state.name_order_store.borrow();
        if store.get_order(caller).is_some() {
            return Err(ICNSError::PendingOrder);
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

pub fn encode_metrics(w: &mut MetricsEncoder<Vec<u8>>) -> std::io::Result<()> {
    let service = RegistrarService::new();
    let now = api::time();
    let stats = service.get_stats(now);
    for (status, count) in stats.user_quota_order_count.iter() {
        w.encode_gauge(
            format!("icnaming_registrar_quota_order_status_{}", status).as_str(),
            *count as f64,
            format!("Number of quota orders with status {}", status).as_str(),
        )?;
    }
    for (status, count) in stats.user_name_order_count_by_status.iter() {
        w.encode_gauge(
            format!("icnaming_registrar_name_order_status_{}", status).as_str(),
            *count as f64,
            format!("Number of name orders with status {}", status).as_str(),
        )?;
    }
    for (t, count) in stats.user_quota_count.iter() {
        if count > &0u64 {
            w.encode_gauge(
                format!("icnaming_registrar_quota_type_{}", t).as_str(),
                *count as f64,
                format!("Number of quotas with type {}", t).as_str(),
            )?;
        }
    }
    w.encode_gauge(
        "icnaming_registrar_user_count",
        stats.user_count as f64,
        "Number of users",
    )?;
    w.encode_gauge(
        "icnaming_registrar_registrations_count",
        stats.registration_count as f64,
        "Number of registrations",
    )?;
    w.encode_gauge(
        "icnaming_registrar_last_xdr_permyriad_per_icp",
        stats.last_xdr_permyriad_per_icp as f64,
        "Last XDR permyriad per ICP",
    )?;
    w.encode_gauge(
        "icnaming_registrar_last_timestamp_seconds_xdr_permyriad_per_icp",
        stats.last_timestamp_seconds_xdr_permyriad_per_icp as f64,
        "Last timestamp seconds XDR permyriad per ICP",
    )?;
    w.encode_counter(
        "icnaming_registrar_name_order_placed_count",
        stats.name_order_placed_count as f64,
        "Number of name orders placed",
    )?;
    w.encode_counter(
        "icnaming_registrar_name_order_paid_count",
        stats.name_order_paid_count as f64,
        "Number of name orders paid",
    )?;
    w.encode_counter(
        "icnaming_registrar_name_order_cancelled_count",
        stats.name_order_cancelled_count as f64,
        "Number of name orders cancelled",
    )?;
    w.encode_counter(
        "icnaming_registrar_new_registered_name_count",
        stats.new_registered_name_count as f64,
        "Number of new registered names",
    )?;
    w.encode_gauge(
        "icnaming_registrar_payment_version",
        stats.payment_version as f64,
        "Payment version synced",
    )?;
    w.encode_gauge(
        "icnaming_registrar_cycles_balance",
        stats.cycles_balance as f64,
        "Cycles balance",
    )?;

    Ok(())
}

#[derive(CandidType, Deserialize, Default)]
pub struct Stats {
    cycles_balance: u64,
    user_count: u64,
    user_quota_order_count: HashMap<String, u64>,
    user_quota_count: HashMap<String, u64>,
    user_name_order_count_by_status: HashMap<String, u64>,
    registration_count: u64,
    last_xdr_permyriad_per_icp: u64,
    last_timestamp_seconds_xdr_permyriad_per_icp: u64,
    name_order_placed_count: u64,
    name_order_paid_count: u64,
    name_order_cancelled_count: u64,
    new_registered_name_count: u64,
    payment_version: u64,
    name_lock_count: u64,
}

#[cfg(test)]
mod tests;

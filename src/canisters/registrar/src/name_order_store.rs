use std::collections::{HashMap, HashSet};

use candid::{decode_args, encode_args, CandidType, Deserialize, Nat, Principal};
use getset::{Getters, Setters};
use log::debug;

use common::icnaming_ledger_types::{PaymentAccountId, PaymentId};
use common::state::StableState;

use crate::quota_order_store::PaymentMemo;
use crate::user_quota_store::QuotaType;

#[derive(Clone, Debug, Deserialize, CandidType, Eq, PartialEq, Hash)]
pub enum NameOrderStatus {
    New,
    Done,
    WaitingToRefund,
    Canceled,
}

#[derive(Getters, Setters)]
#[getset(get = "pub")]
#[derive(CandidType, Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct NameOrder {
    name: String,
    years: u32,
    created_user: Principal,
    order_status: NameOrderStatus,
    quota_type: QuotaType,
    payment_memo: PaymentMemo,
    payment_id: PaymentId,
    payment_account_id: PaymentAccountId,
    price_icp_in_e8s: Nat,
}

#[derive(Deserialize, CandidType)]
pub struct GetNameOrderResponse {
    name: String,
    years: u32,
    created_user: Principal,
    status: NameOrderStatus,
    quota_type: QuotaType,
    payment_memo: PaymentMemo,
    payment_id: PaymentId,
    payment_account_id: PaymentAccountId,
    price_icp_in_e8s: Nat,
}

impl From<&NameOrder> for GetNameOrderResponse {
    fn from(name_order: &NameOrder) -> Self {
        GetNameOrderResponse {
            name: name_order.name.clone(),
            years: name_order.years,
            created_user: name_order.created_user.clone(),
            status: name_order.order_status.clone(),
            payment_memo: name_order.payment_memo.clone(),
            price_icp_in_e8s: name_order.price_icp_in_e8s.clone(),
            payment_id: name_order.payment_id.clone(),
            quota_type: name_order.quota_type.clone(),
            payment_account_id: name_order.payment_account_id.clone(),
        }
    }
}

#[derive(Default)]
pub struct NameOrderStore {
    name_orders: HashMap<Principal, NameOrder>,
    name_orders_payment_id_map: HashMap<PaymentId, Principal>,
    handling_payment_ids: HashSet<PaymentId>,
}

impl StableState for NameOrderStore {
    fn encode(&self) -> Vec<u8> {
        encode_args((
            &self.name_orders,
            &self.name_orders_payment_id_map,
            &self.handling_payment_ids,
        ))
        .unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let (name_orders, name_orders_payment_id_map, handling_payment_ids): (
            HashMap<Principal, NameOrder>,
            HashMap<PaymentId, Principal>,
            HashSet<PaymentId>,
        ) = decode_args(&bytes).unwrap();

        Ok(NameOrderStore {
            name_orders,
            name_orders_payment_id_map,
            handling_payment_ids,
        })
    }
}

impl NameOrderStore {
    pub fn has_name_order(&self, principal: &Principal) -> bool {
        self.name_orders.contains_key(principal)
    }

    pub fn add_name_order(
        &mut self,
        created_user: &Principal,
        name: &str,
        years: u32,
        payment_memo: &PaymentMemo,
        price_icp_in_e8s: Nat,
        payment_id: &PaymentId,
        payment_account_id: &PaymentAccountId,
        quota_type: &QuotaType,
    ) {
        self.name_orders.insert(
            created_user.clone(),
            NameOrder {
                name: name.to_string(),
                years,
                created_user: created_user.clone(),
                order_status: NameOrderStatus::New,
                payment_memo: payment_memo.clone(),
                price_icp_in_e8s,
                payment_id: payment_id.clone(),
                quota_type: quota_type.clone(),
                payment_account_id: payment_account_id.clone(),
            },
        );
        self.name_orders_payment_id_map
            .insert(payment_id.clone(), created_user.clone());
    }

    pub fn get_order(&self, principal: &Principal) -> Option<&NameOrder> {
        self.name_orders.get(principal)
    }

    pub fn get_order_by_payment_id(&self, payment_id: &PaymentId) -> Option<&NameOrder> {
        self.name_orders_payment_id_map
            .get(payment_id)
            .and_then(|principal| self.name_orders.get(principal))
    }

    pub fn remove_name_order(&mut self, principal: &Principal) {
        debug!("remove_name_order: principal: {}", principal);
        let order = self.name_orders.remove(principal).unwrap();
        self.name_orders_payment_id_map.remove(&order.payment_id);
    }

    pub fn cancel_name_order(&mut self, principal: &Principal) {
        if let Some(name_order) = self.name_orders.get_mut(principal) {
            name_order.order_status = NameOrderStatus::Canceled;
        }
    }

    pub fn waiting_to_refund(&mut self, principal: &Principal) {
        if let Some(name_order) = self.name_orders.get_mut(principal) {
            name_order.order_status = NameOrderStatus::WaitingToRefund;
        }
    }

    pub fn get_need_verify_payment_ids(&self) -> Vec<PaymentId> {
        self.name_orders
            .values()
            .filter(|name_order| name_order.order_status == NameOrderStatus::New)
            .map(|name_order| name_order.payment_id.clone())
            .collect()
    }

    pub fn get_all_orders(&self) -> &HashMap<Principal, NameOrder> {
        &self.name_orders
    }

    pub fn add_handling_payment_id(&mut self, payment_id: PaymentId) -> Result<(), String> {
        if self.handling_payment_ids.contains(&payment_id) {
            Err(format!("payment_id: {} already exists", payment_id))
        } else {
            self.handling_payment_ids.insert(payment_id);
            Ok(())
        }
    }

    pub fn remove_handling_payment_id(&mut self, payment_id: PaymentId) -> Result<(), String> {
        if !self.handling_payment_ids.contains(&payment_id) {
            Err(format!("payment_id: {} not exists", payment_id))
        } else {
            self.handling_payment_ids.remove(&payment_id);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests;

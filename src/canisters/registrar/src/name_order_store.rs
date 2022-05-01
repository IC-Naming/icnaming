use std::collections::{HashMap, HashSet};

use candid::{decode_args, encode_args, CandidType, Deserialize, Nat, Principal};
use common::constants::{
    EXPIRE_TIME_OF_NAME_ORDER_AVAILABILITY_CHECK_IN_NS, EXPIRE_TIME_OF_NAME_ORDER_IN_NS,
};
use common::naming::FirstLevelName;
use log::debug;

use common::state::StableState;

#[derive(Clone, Debug, Deserialize, CandidType, Eq, PartialEq, Hash)]
pub enum NameOrderStatus {
    New,
    Done,
    WaitingToRefund,
    Canceled,
}

#[derive(CandidType, Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct NameOrder {
    name: String,
    years: u32,
    created_user: Principal,
    order_status: NameOrderStatus,
    created_at: u64,
    price_icp_in_e8s: Nat,
}

impl NameOrder {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn years(&self) -> u32 {
        self.years
    }
    pub fn created_user(&self) -> Principal {
        self.created_user
    }
    pub fn order_status(&self) -> &NameOrderStatus {
        &self.order_status
    }
    pub fn created_at(&self) -> u64 {
        self.created_at
    }
    pub fn price_icp_in_e8s(&self) -> &Nat {
        &self.price_icp_in_e8s
    }
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
    pub fn set_years(&mut self, years: u32) {
        self.years = years;
    }
    pub fn set_created_user(&mut self, created_user: Principal) {
        self.created_user = created_user;
    }
    pub fn set_order_status(&mut self, order_status: NameOrderStatus) {
        self.order_status = order_status;
    }
    pub fn set_created_at(&mut self, created_at: u64) {
        self.created_at = created_at;
    }
    pub fn set_price_icp_in_e8s(&mut self, price_icp_in_e8s: Nat) {
        self.price_icp_in_e8s = price_icp_in_e8s;
    }
}

#[derive(Deserialize, CandidType)]
pub struct GetNameOrderResponse {
    name: String,
    years: u32,
    created_user: Principal,
    created_at: u64,
    status: NameOrderStatus,
    price_icp_in_e8s: Nat,
}

impl From<&NameOrder> for GetNameOrderResponse {
    fn from(name_order: &NameOrder) -> Self {
        GetNameOrderResponse {
            name: name_order.name.clone(),
            years: name_order.years,
            created_at: name_order.created_at,
            created_user: name_order.created_user.clone(),
            status: name_order.order_status.clone(),
            price_icp_in_e8s: name_order.price_icp_in_e8s.clone(),
        }
    }
}

#[derive(Default)]
pub struct NameOrderStore {
    name_orders: HashMap<Principal, NameOrder>,
    handling_names: HashSet<String>,
}

impl StableState for NameOrderStore {
    fn encode(&self) -> Vec<u8> {
        encode_args((&self.name_orders, &self.handling_names)).unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let (name_orders, handling_names): (HashMap<Principal, NameOrder>, HashSet<String>) =
            decode_args(&bytes).unwrap();

        Ok(NameOrderStore {
            name_orders,
            handling_names,
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
        price_icp_in_e8s: Nat,
        now: u64,
    ) {
        self.name_orders.insert(
            created_user.clone(),
            NameOrder {
                name: name.to_string(),
                years,
                created_at: now,
                created_user: created_user.clone(),
                order_status: NameOrderStatus::New,
                price_icp_in_e8s,
            },
        );
    }

    pub fn get_order(&self, principal: &Principal) -> Option<&NameOrder> {
        self.name_orders.get(principal)
    }

    pub fn remove_name_order(&mut self, principal: &Principal) {
        debug!("remove_name_order: principal: {}", principal);
        let order = self.name_orders.remove(principal).unwrap();
    }

    pub fn cancel_name_order(&mut self, principal: &Principal) {
        if let Some(name_order) = self.name_orders.get_mut(principal) {
            name_order.order_status = NameOrderStatus::Canceled;
        }
    }

    pub fn get_all_orders(&self) -> &HashMap<Principal, NameOrder> {
        &self.name_orders
    }

    pub fn add_handling_name(&mut self, name: &FirstLevelName) -> Result<(), String> {
        if self.handling_names.contains(name.0.get_name()) {
            return Err(format!("name: {} is already handling", name));
        } else {
            self.handling_names.insert(name.0.get_name().clone());
            Ok(())
        }
    }

    pub fn remove_handling_name(&mut self, name: &FirstLevelName) -> Result<(), String> {
        if self.handling_names.contains(name.0.get_name()) {
            self.handling_names.remove(name.0.get_name());
            Ok(())
        } else {
            Err(format!("name: {} is not handling", name))
        }
    }

    pub fn get_expired_quota_order_user_principals(&self, now: u64) -> Vec<Principal> {
        let expired_time = now - EXPIRE_TIME_OF_NAME_ORDER_IN_NS;
        self.name_orders
            .iter()
            .filter(|(_, order)| {
                order.order_status == NameOrderStatus::New && order.created_at < expired_time
            })
            .map(|(user, _)| user.clone())
            .collect()
    }

    pub fn get_need_to_be_check_name_availability_principals(&self, now: u64) -> Vec<Principal> {
        let start_time = now - EXPIRE_TIME_OF_NAME_ORDER_IN_NS;
        let end_time = now - EXPIRE_TIME_OF_NAME_ORDER_AVAILABILITY_CHECK_IN_NS;
        self.name_orders
            .iter()
            .filter(|(_, order)| {
                order.order_status == NameOrderStatus::New
                    && order.created_at >= start_time
                    && order.created_at < end_time
            })
            .map(|(user, _)| user.clone())
            .collect()
    }
}

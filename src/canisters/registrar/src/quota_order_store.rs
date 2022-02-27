use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use candid::{decode_args, encode_args, CandidType, Deserialize, Nat, Principal};
use getset::{Getters, Setters};
use log::info;

use common::icnaming_ledger_types::{PaymentAccountId, PaymentId};
use common::state::StableState;

use crate::user_quota_store::QuotaType;

pub type QuotaOrderId = u64;

#[derive(Deserialize, CandidType, Clone, Eq, PartialEq, Debug, Hash)]
pub enum QuotaOrderStatus {
    New,
    Done,
    Canceled,
}

pub type QuotaOrderDetails = HashMap<Principal, HashMap<QuotaType, u32>>;

#[derive(CandidType, Clone, Eq, PartialEq, Debug)]
pub struct PlaceOrderOutput {
    pub order: GetOrderOutput,
}

#[derive(Deserialize, CandidType, Hash, Clone, Eq, PartialEq, Debug)]
pub enum PaymentType {
    ICP,
}

#[derive(Deserialize, CandidType, Clone, Eq, PartialEq, Debug)]
pub struct ICPBlockHeight(pub u64);

#[derive(Deserialize, CandidType, Clone, Eq, PartialEq, Debug)]
pub enum PaymentStatus {
    New,
    Verified,
}

#[derive(Deserialize, CandidType, Clone, Eq, PartialEq, Debug)]
pub enum PaymentMemo {
    ICP(ICPMemo),
}

#[derive(Deserialize, CandidType, Clone, Eq, PartialEq, Debug)]
pub enum PaidData {
    ICP(ICPBlockHeight),
}

#[derive(Getters, Setters)]
#[getset(get = "pub")]
#[derive(CandidType, Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct QuotaOrder {
    id: QuotaOrderId,
    created_user: Principal,
    details: QuotaOrderDetails,
    created_at: u64,
    status: QuotaOrderStatus,
    payment: QuotaOrderPayment,
    /// time of all payments verified
    paid_at: Option<u64>,
    canceled_at: Option<u64>,
}

#[derive(CandidType, Clone, Eq, PartialEq, Debug)]
pub struct GetOrderOutput {
    pub id: QuotaOrderId,
    pub created_user: Principal,
    pub details: QuotaOrderDetails,
    pub created_at: u64,
    pub status: QuotaOrderStatus,
    pub payment: QuotaOrderPayment,
    pub paid_at: Option<u64>,
    pub canceled_at: Option<u64>,
}

#[derive(CandidType, Clone, Eq, PartialEq, Debug)]
pub struct PaidOrderOutput {
    pub order: GetOrderOutput,
    pub is_all_paid: bool,
}

impl From<&QuotaOrder> for GetOrderOutput {
    fn from(order: &QuotaOrder) -> Self {
        GetOrderOutput {
            id: order.id.clone(),
            created_user: order.created_user.clone(),
            details: order.details.clone(),
            created_at: order.created_at.clone(),
            status: order.status.clone(),
            paid_at: order.paid_at.clone(),
            canceled_at: order.canceled_at.clone(),
            payment: order.payment.clone(),
        }
    }
}

impl QuotaOrder {
    pub fn is_paid(&self) -> bool {
        self.paid_at.is_some()
    }

    pub fn verified_payment(&mut self, now: u64) {
        self.payment.verified(now);
        self.paid_at = Some(now);
        self.status = QuotaOrderStatus::Done;
    }

    pub fn cancel(&mut self, now: u64) {
        self.canceled_at = Some(now);
        self.status = QuotaOrderStatus::Canceled;
    }
}

pub type QuotaOrderRef = Rc<RefCell<QuotaOrder>>;

#[derive(Deserialize, CandidType, Clone, Eq, PartialEq, Debug)]
pub struct ICPMemo(pub u64);

#[derive(Getters, Setters, CandidType, Deserialize, Clone, Eq, PartialEq, Debug)]
#[getset(get = "pub")]
pub struct QuotaOrderPayment {
    payment_id: PaymentId,
    payment_type: PaymentType,
    payment_memo: PaymentMemo,
    payment_account_id: PaymentAccountId,
    payment_status: PaymentStatus,
    amount: Nat,
    verified_at: Option<u64>,
}

impl QuotaOrderPayment {
    pub fn new(
        payment_id: PaymentId,
        payment_type: PaymentType,
        amount: Nat,
        memo: PaymentMemo,
        account_id: PaymentAccountId,
    ) -> QuotaOrderPayment {
        QuotaOrderPayment {
            payment_id,
            payment_type,
            payment_memo: memo,
            payment_account_id: account_id,
            payment_status: PaymentStatus::New,
            amount,
            verified_at: None,
        }
    }

    pub fn verified(&mut self, verified_at: u64) {
        self.payment_status = PaymentStatus::Verified;
        self.verified_at = Some(verified_at);
    }
}

#[derive(Getters, Setters, Default)]
#[getset(get = "pub")]
pub struct QuotaOrderStore {
    user_orders: HashMap<Principal, QuotaOrderRef>,
    all_orders: HashMap<QuotaOrderId, QuotaOrderRef>,
    payment_orders: HashMap<PaymentId, QuotaOrderRef>,
    next_id: QuotaOrderId,
}

impl StableState for QuotaOrderStore {
    fn encode(&self) -> Vec<u8> {
        let orders: Vec<(QuotaOrderId, QuotaOrder)> = self
            .all_orders
            .iter()
            .map(|(id, order)| (id.clone(), order.borrow().clone()))
            .collect();
        encode_args((&orders, &self.next_id)).unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        let (orders, next_id): (Vec<(QuotaOrderId, QuotaOrder)>, QuotaOrderId) =
            decode_args(&bytes).unwrap();

        let mut user_orders = HashMap::new();
        let mut all_orders = HashMap::new();
        let mut payment_orders = HashMap::new();
        for (order_id, order) in orders {
            let payment_id = order.payment.payment_id.clone();
            let create_user = order.created_user.clone();
            let order_ref = Rc::new(RefCell::new(order));
            user_orders.insert(create_user, order_ref.clone());
            all_orders.insert(order_id.clone(), order_ref.clone());
            payment_orders.insert(payment_id, order_ref);
        }
        Ok(QuotaOrderStore {
            user_orders,
            all_orders,
            payment_orders,
            next_id,
        })
    }
}

impl QuotaOrderStore {
    pub fn new() -> QuotaOrderStore {
        QuotaOrderStore {
            user_orders: HashMap::new(),
            all_orders: HashMap::new(),
            payment_orders: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn add_order(
        &mut self,
        created_user: Principal,
        details: QuotaOrderDetails,
        created_at: u64,
        payment: QuotaOrderPayment,
    ) -> QuotaOrderId {
        assert!(self.user_orders.get(&created_user).is_none());
        self.next_id += 1;
        let id = self.next_id;
        let payment_id = payment.payment_id;
        let order = QuotaOrder {
            id: id.clone(),
            created_user,
            details,
            created_at,
            paid_at: None,
            canceled_at: None,
            payment,
            status: QuotaOrderStatus::New,
        };
        let order_ref = Rc::new(RefCell::new(order));
        self.user_orders.insert(created_user, order_ref.clone());
        self.all_orders.insert(id, order_ref.clone());
        self.payment_orders.insert(payment_id, order_ref);
        id
    }

    pub fn has_pending_order(&self, principal: &Principal) -> bool {
        self.user_orders.get(principal).is_some()
    }

    pub fn get_order(&self, principal: &Principal) -> Option<&QuotaOrderRef> {
        self.user_orders.get(principal)
    }

    pub fn get_order_by_payment_id(&self, payment_id: &PaymentId) -> Option<&QuotaOrderRef> {
        self.payment_orders.get(payment_id)
    }

    pub fn remove_order_by_principal(&mut self, user: &Principal) -> Option<QuotaOrderRef> {
        if let Some(order) = self.user_orders.remove(user) {
            let returning = order.clone();
            let order_ref = order.borrow();
            info!("remove order, user: {:?}", user.clone());
            self.all_orders.remove(&order_ref.id).unwrap();
            self.payment_orders
                .remove(&order_ref.payment.payment_id)
                .unwrap();
            Some(returning)
        } else {
            None
        }
    }

    pub fn get_all_orders(&self) -> &HashMap<QuotaOrderId, QuotaOrderRef> {
        &self.all_orders
    }
}

#[cfg(test)]
mod tests;

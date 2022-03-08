use candid::Principal;
use rstest::*;

use test_common::ic_api::init_test;
use test_common::user::*;

use crate::quota_order_store::{
    ICPMemo, PaymentMemo, PaymentType, QuotaOrderPayment, QuotaOrderStore,
};

use super::*;

#[fixture]
fn empty_quote_order_manager(_init_test: ()) -> QuotaOrderStore {
    QuotaOrderStore::new()
}

#[fixture]
fn quote_order_manager_with_one_order(
    _empty_quote_order_manager: QuotaOrderStore,
    mock_user1: Principal,
    mock_user2: Principal,
    mock_user3: Principal,
    mock_now: u64,
) -> QuotaOrderStore {
    let mut manager = QuotaOrderStore::new();
    let mut details = HashMap::new();
    let mut quota_items1 = HashMap::new();
    quota_items1.insert(QuotaType::LenGte(1), 6);
    quota_items1.insert(QuotaType::LenEq(2), 13);
    details.insert(mock_user2.clone(), quota_items1);
    let mut quota_items2 = HashMap::new();
    quota_items2.insert(QuotaType::LenGte(1), 3);
    details.insert(mock_user3.clone(), quota_items2);
    let payment = QuotaOrderPayment::new(
        123,
        PaymentType::ICP,
        Nat::from(1234),
        PaymentMemo::ICP(ICPMemo(234)),
        vec![],
    );

    // act
    manager.add_order(mock_user1.clone(), details.clone(), mock_now, payment);
    manager
}

#[rstest]
fn test_add_order(
    mut empty_quote_order_manager: QuotaOrderStore,
    mock_user1: Principal,
    mock_user2: Principal,
    mock_user3: Principal,
    mock_now: u64,
) {
    let mut details = HashMap::new();
    let mut quota_items1 = HashMap::new();
    quota_items1.insert(QuotaType::LenGte(1), 6);
    quota_items1.insert(QuotaType::LenEq(2), 13);
    details.insert(mock_user2.clone(), quota_items1);
    let mut quota_items2 = HashMap::new();
    quota_items2.insert(QuotaType::LenGte(1), 3);
    details.insert(mock_user3.clone(), quota_items2);
    let payment = QuotaOrderPayment::new(
        123,
        PaymentType::ICP,
        Nat::from(1234),
        PaymentMemo::ICP(ICPMemo(234)),
        vec![],
    );

    // act
    let order_id = empty_quote_order_manager.add_order(
        mock_user1.clone(),
        details.clone(),
        mock_now,
        payment.clone(),
    );

    // assert
    assert_eq!(order_id, 1);
    let order_ref = empty_quote_order_manager.get_order(&mock_user1).unwrap();
    let order = order_ref.borrow();
    assert_eq!(order.id(), &order_id);
    assert_eq!(order.details(), &details);
    assert_eq!(order.payment(), &payment);
}

#[rstest]
#[should_panic]
fn test_add_order_failed_with_pending(
    mut quote_order_manager_with_one_order: QuotaOrderStore,
    mock_user1: Principal,
) {
    quote_order_manager_with_one_order.add_order(
        mock_user1.clone(),
        HashMap::new(),
        0,
        QuotaOrderPayment::new(
            123,
            PaymentType::ICP,
            Nat::from(1234),
            PaymentMemo::ICP(ICPMemo(234)),
            vec![],
        ),
    );
}

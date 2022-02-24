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

mod memory {
    use log::info;

    use crate::quota_order_store::ICPMemo;

    use super::*;

    #[rstest]
    fn test_name_order_many(_init_test: ()) {
        let counts = vec![1u32, 10, 100, 1_000, 10_000];

        // run each count and record the size of the store
        let mut sizes = vec![];
        for count in counts.iter() {
            let mut store = QuotaOrderStore::default();

            for index in 0..*count {
                let created_user = mock_user(index);
                let details = HashMap::new();
                let created_at = u64::MAX - 1;
                let payment_id = 123;
                let payment_type = PaymentType::ICP;
                let amount = Nat::from(1234u64 * 10_000_000u64);
                let memo = PaymentMemo::ICP(ICPMemo(234));
                let account_id = mock_account_id(index, 1);
                let payment =
                    QuotaOrderPayment::new(payment_id, payment_type, amount, memo, account_id);
                store.add_order(
                    created_user.clone(),
                    details.clone(),
                    created_at,
                    payment.clone(),
                );
            }

            let encode_state = store.encode();
            sizes.push(encode_state.len());
        }

        info!("add quota orders done, print sizes");
        // info the sizes with count
        for (count, size) in counts.iter().zip(sizes.iter()) {
            info!(
                "{} quota orders: {} bytes, average {} bytes each quota order",
                count,
                size,
                size / *count as usize
            );
        }

        // add quota orders done, print sizes
        // 1 quota orders: 342 bytes, average 342 bytes each quota order
        // 10 quota orders: 1683 bytes, average 168 bytes each quota order
        // 100 quota orders: 15093 bytes, average 150 bytes each quota order
        // 1000 quota orders: 149194 bytes, average 149 bytes each quota order
        // 10000 quota orders: 1490194 bytes, average 149 bytes each quota order
        // 50000 quota orders: 7450195 bytes, average 149 bytes each quota order
        // 100000 quota orders: 14900195 bytes, average 149 bytes each quota order
    }
}

use std::collections::HashSet;
use std::str::FromStr;

use crate::{get_named_get_canister_id, CANISTER_NAME_REGISTRAR};
use ledger_canister::ICPTs;
use maplit::hashset;

use super::*;

const TEST_ACCOUNT_1: &str = "h4a5i-5vcfo-5rusv-fmb6m-vrkia-mjnkc-jpoow-h5mam-nthnm-ldqlr-bqe";
const TEST_ACCOUNT_2: &str = "bngem-gzprz-dtr6o-xnali-fgmfi-fjgpb-rya7j-x2idk-3eh6u-4v7tx-hqe";
const TEST_ACCOUNT_3: &str = "347of-sq6dc-h53df-dtzkw-eama6-hfaxk-a7ghn-oumsd-jf2qy-tqvqc-wqe";
const TEST_ACCOUNT_4: &str = "zrmyx-sbrcv-rod5f-xyd6k-letwb-tukpj-edhrc-sqash-lddmc-7qypw-yqe";
const TEST_ACCOUNT_5: &str = "2fzwl-cu3hl-bawo2-idwrw-7yygk-uccms-cbo3a-c6kqt-lnk3j-mewg3-hae";
const TEST_ACCOUNT_6: &str = "4gb44-uya57-c2v6u-fcz5v-qrpwl-wqkmf-o3fd3-esjio-kpysm-r5xxh-fqe";
const TEST_NOW_TIMESTAMP_NANOS: u64 = 1643224478 * 1_000_000_000;

mod add_payment {
    use super::*;

    #[test]
    fn add_payment_success() {
        let mut store = setup_test_store();
        let request = AddPaymentRequest {
            amount: ICPTs::from_icpts(10).unwrap(),
            created_remark: "test remark".to_string(),
        };
        let now = TimeStamp {
            timestamp_nanos: 100,
        };

        // act
        let caller = get_allow_add_payment_principal();
        let result = store.add_payment(request.clone(), caller.clone(), now);

        // assert
        assert_eq!(result.payment_id, 2u64);
        assert_eq!(result.payment_id, result.memo.0);
        assert_eq!(store.next_payment_id, result.payment_id + 1);

        let payment = store.payments.get(&result.payment_id).unwrap();
        assert_eq!(payment.id, result.payment_id);
        assert_eq!(payment.created_by, caller);
        assert_eq!(payment.created_remark, request.created_remark);
        assert_eq!(payment.amount, request.amount);
        assert_eq!(payment.received_amount.get_e8s(), 0u64);
        assert_eq!(payment.created_at, now);
        assert_eq!(payment.paid_at.is_none(), true);
        assert_eq!(payment.transactions_last5.len(), 0);
        assert_eq!(payment.block_heights.len(), 0);
        assert_eq!(payment.payment_status, PaymentStatus::New);
    }

    #[test]
    #[should_panic]
    fn add_payment_failed_not_allow_canister_id() {
        let mut store = setup_test_store();
        let request = AddPaymentRequest {
            amount: ICPTs::from_icpts(10).unwrap(),
            created_remark: "test remark".to_string(),
        };
        let now = TimeStamp {
            timestamp_nanos: 100,
        };
        let caller = PrincipalId::from_str(TEST_ACCOUNT_1).unwrap();

        // act
        store.add_payment(request, caller.clone(), now);
    }

    #[test]
    #[should_panic]
    fn add_payment_failed_remark_too_long() {
        let mut store = setup_test_store();
        let request = AddPaymentRequest {
            amount: ICPTs::from_icpts(10).unwrap(),
            // remark is too long, which is greater than the max length of remark
            created_remark: "test remark".repeat(100),
        };
        let now = TimeStamp {
            timestamp_nanos: 100,
        };
        let caller = PrincipalId::from_str(TEST_ACCOUNT_2).unwrap();

        // act
        store.add_payment(request, caller.clone(), now);
    }
}

mod verify_payment {
    use super::*;

    #[test]
    fn verify_payment_need_more_at_first() {
        let mut store = setup_test_store();
        let request = AddPaymentRequest {
            amount: ICPTs::from_icpts(10).unwrap(),
            created_remark: "test remark".to_string(),
        };
        let now = TimeStamp {
            timestamp_nanos: 100,
        };
        let result = store.add_payment(request.clone(), get_allow_add_payment_principal(), now);

        // act
        let payment_id = result.payment_id;
        let result = store.verify_payment(VerifyPaymentRequest { payment_id });

        // assert
        match result {
            VerifyPaymentResponse::NeedMore {
                received_amount,
                amount,
            } => {
                assert_eq!(received_amount.get_e8s(), 0u64);
                assert_eq!(amount, ICPTs::from_icpts(10).unwrap());
            }
            _ => panic!("unexpected result"),
        }
        assert_eq!(
            store.payments.get(&payment_id).unwrap().payment_status,
            PaymentStatus::New
        );
    }

    #[test]
    fn verify_payment_need_more_with_some_paid() {
        let mut store = setup_test_store();
        let request = AddPaymentRequest {
            amount: ICPTs::from_icpts(10).unwrap(),
            created_remark: "test remark".to_string(),
        };
        let now = TimeStamp {
            timestamp_nanos: 100,
        };
        let result = store.add_payment(request.clone(), get_allow_add_payment_principal(), now);
        {
            let transfer = Send {
                from: get_some_user_account_id(),
                fee: ICPTs::from_icpts(1).unwrap(),
                amount: ICPTs::from_icpts(1).unwrap(),
                to: get_icnaming_ledger_account_id(),
            };
            let re = store.try_sync_transaction(transfer, result.memo, 4u64, now);
            assert_eq!(re.is_ok(), true);
        }

        // act
        let payment_id = result.payment_id;
        let result = store.verify_payment(VerifyPaymentRequest { payment_id });

        // assert
        match result {
            VerifyPaymentResponse::NeedMore {
                received_amount,
                amount,
            } => {
                assert_eq!(received_amount, ICPTs::from_icpts(1).unwrap());
                assert_eq!(amount, ICPTs::from_icpts(10).unwrap());
            }
            _ => panic!("unexpected result"),
        }

        assert_eq!(
            store.payments.get(&payment_id).unwrap().payment_status,
            PaymentStatus::NeedMore
        );
    }

    #[test]
    fn verify_payment_paid_with_all_paid() {
        let mut store = setup_test_store();
        let request = AddPaymentRequest {
            amount: ICPTs::from_icpts(10).unwrap(),
            created_remark: "test remark".to_string(),
        };
        let now = TimeStamp {
            timestamp_nanos: 100,
        };
        let result = store.add_payment(request.clone(), get_allow_add_payment_principal(), now);
        {
            let transfer = Send {
                from: get_some_user_account_id(),
                fee: ICPTs::from_icpts(1).unwrap(),
                amount: ICPTs::from_icpts(10).unwrap(),
                to: get_icnaming_ledger_account_id(),
            };
            let re = store.try_sync_transaction(transfer, result.memo, 4u64, now);
            assert_eq!(re.is_ok(), true);
        }

        // act
        let payment_id = result.payment_id;
        let result = store.verify_payment(VerifyPaymentRequest { payment_id });

        // assert
        match result {
            VerifyPaymentResponse::Paid { paid_at } => {
                assert_eq!(paid_at, now);
            }
            _ => panic!("unexpected result"),
        }
        assert_eq!(
            store.payments.get(&payment_id).unwrap().payment_status,
            PaymentStatus::Paid
        );
    }

    #[test]
    fn verify_payment_not_found() {
        let mut store = setup_test_store();
        let request = AddPaymentRequest {
            amount: ICPTs::from_icpts(10).unwrap(),
            created_remark: "test remark".to_string(),
        };
        let now = TimeStamp {
            timestamp_nanos: 100,
        };
        let result = store.add_payment(request.clone(), get_allow_add_payment_principal(), now);

        // act
        let result = store.verify_payment(VerifyPaymentRequest { payment_id: 123u64 });

        // assert
        match result {
            VerifyPaymentResponse::PaymentNotFound {} => {
                // ok
            }
            _ => panic!("unexpected result"),
        }
    }
}

mod try_sync_transaction {
    use super::*;

    #[test]
    fn try_sync_transaction_success_some_paid() {
        let mut store = setup_test_store();
        let now = TimeStamp {
            timestamp_nanos: 100,
        };
        let request = AddPaymentRequest {
            amount: ICPTs::from_icpts(10).unwrap(),
            created_remark: "test remark".to_string(),
        };
        let payment_result =
            store.add_payment(request.clone(), get_allow_add_payment_principal(), now);
        let transfer = Send {
            from: get_some_user_account_id(),
            fee: ICPTs::from_icpts(1).unwrap(),
            amount: ICPTs::from_icpts(1).unwrap(),
            to: get_icnaming_ledger_account_id(),
        };

        // act
        let result = store.try_sync_transaction(transfer, payment_result.memo, 4u64, now);

        // assert
        assert_eq!(result.is_ok(), true);
        let payment = store.payments.get(&payment_result.payment_id).unwrap();
        assert_eq!(payment.transactions_last5.len(), 1);
        assert_eq!(payment.block_heights.len(), 1);
        assert_eq!(payment.received_amount, ICPTs::from_icpts(1).unwrap());
        assert_eq!(payment.amount, ICPTs::from_icpts(10).unwrap());
        assert_eq!(payment.paid_at, None);
    }

    #[test]
    fn try_sync_transaction_success_some_all_paid() {
        let mut store = setup_test_store();
        let request = AddPaymentRequest {
            amount: ICPTs::from_icpts(10).unwrap(),
            created_remark: "test remark".to_string(),
        };
        let now = TimeStamp {
            timestamp_nanos: 100,
        };
        let payment_result =
            store.add_payment(request.clone(), get_allow_add_payment_principal(), now);
        let transfer = Send {
            from: get_some_user_account_id(),
            fee: ICPTs::from_icpts(1).unwrap(),
            amount: ICPTs::from_icpts(10).unwrap(),
            to: get_icnaming_ledger_account_id(),
        };

        // act
        let result = store.try_sync_transaction(transfer, payment_result.memo, 4u64, now);

        // assert
        assert_eq!(result.is_ok(), true);
        let payment = store.payments.get(&payment_result.payment_id).unwrap();
        assert_eq!(payment.transactions_last5.len(), 1);
        assert_eq!(payment.block_heights.len(), 1);
        assert_eq!(payment.received_amount, ICPTs::from_icpts(10).unwrap());
        assert_eq!(payment.amount, ICPTs::from_icpts(10).unwrap());
        assert_eq!(payment.paid_at, Some(now));
    }

    #[test]
    fn try_sync_transaction_failed_memo_not_found() {
        let mut store = setup_test_store();
        let request = AddPaymentRequest {
            amount: ICPTs::from_icpts(10).unwrap(),
            created_remark: "test remark".to_string(),
        };
        let now = TimeStamp {
            timestamp_nanos: 100,
        };
        let payment_result =
            store.add_payment(request.clone(), get_allow_add_payment_principal(), now);
        let transfer = Send {
            from: get_some_user_account_id(),
            fee: ICPTs::from_icpts(1).unwrap(),
            amount: ICPTs::from_icpts(10).unwrap(),
            to: get_icnaming_ledger_account_id(),
        };

        // act
        let result =
            store.try_sync_transaction(transfer, Memo(payment_result.memo.0 + 1), 4u64, now);

        // assert
        assert_eq!(result.is_ok(), true);
        let payment = store.payments.get(&payment_result.payment_id).unwrap();
        assert_eq!(payment.transactions_last5.len(), 0);
        assert_eq!(payment.block_heights.len(), 0);
        assert_eq!(payment.received_amount, ICPTs::from_icpts(0).unwrap());
        assert_eq!(payment.amount, ICPTs::from_icpts(10).unwrap());
        assert_eq!(payment.paid_at, None);
    }

    #[test]
    fn try_sync_transaction_failed_target_id_wrong() {
        let mut store = setup_test_store();
        let request = AddPaymentRequest {
            amount: ICPTs::from_icpts(10).unwrap(),
            created_remark: "test remark".to_string(),
        };
        let now = TimeStamp {
            timestamp_nanos: 100,
        };
        let payment_result =
            store.add_payment(request.clone(), get_allow_add_payment_principal(), now);
        let transfer = Send {
            from: get_some_user_account_id(),
            fee: ICPTs::from_icpts(1).unwrap(),
            amount: ICPTs::from_icpts(10).unwrap(),
            to: get_some_user_account_id(),
        };

        // act
        let result = store.try_sync_transaction(transfer, payment_result.memo, 4u64, now);

        // assert
        assert_eq!(result.is_ok(), true);
        let payment = store.payments.get(&payment_result.payment_id).unwrap();
        assert_eq!(payment.transactions_last5.len(), 0);
        assert_eq!(payment.block_heights.len(), 0);
        assert_eq!(payment.received_amount, ICPTs::from_icpts(0).unwrap());
        assert_eq!(payment.amount, ICPTs::from_icpts(10).unwrap());
        assert_eq!(payment.paid_at, None);
    }
}

mod sync_icp_payment {
    use super::*;

    #[test]
    fn sync_icp_payment_success_some_all_paid() {
        let mut store = setup_test_store();
        let request = AddPaymentRequest {
            amount: ICPTs::from_icpts(10).unwrap(),
            created_remark: "test remark".to_string(),
        };
        let now = TimeStamp {
            timestamp_nanos: 100,
        };
        let payment_result =
            store.add_payment(request.clone(), get_allow_add_payment_principal(), now);
        let transfer = Send {
            from: get_some_user_account_id(),
            fee: ICPTs::from_icpts(1).unwrap(),
            amount: ICPTs::from_icpts(10).unwrap(),
            to: get_icnaming_ledger_account_id(),
        };

        // act
        let result = store.sync_icp_payment(4u64, transfer, payment_result.memo, now);

        // assert
        let payment = store.payments.get(&payment_result.payment_id).unwrap();
        assert_eq!(payment.transactions_last5.len(), 1);
        assert_eq!(payment.block_heights.len(), 1);
        assert_eq!(payment.received_amount, ICPTs::from_icpts(10).unwrap());
        assert_eq!(payment.amount, ICPTs::from_icpts(10).unwrap());
        assert_eq!(payment.paid_at, Some(now));
    }

    #[test]
    fn sync_icp_payment_failed_memo_not_found() {
        let mut store = setup_test_store();
        let request = AddPaymentRequest {
            amount: ICPTs::from_icpts(10).unwrap(),
            created_remark: "test remark".to_string(),
        };
        let now = TimeStamp {
            timestamp_nanos: 100,
        };
        let payment_result =
            store.add_payment(request.clone(), get_allow_add_payment_principal(), now);
        let transfer = Send {
            from: get_some_user_account_id(),
            fee: ICPTs::from_icpts(1).unwrap(),
            amount: ICPTs::from_icpts(10).unwrap(),
            to: get_icnaming_ledger_account_id(),
        };

        // act
        let result = store.sync_icp_payment(4u64, transfer, Memo(payment_result.memo.0 + 1), now);

        // assert
        let payment = store.payments.get(&payment_result.payment_id).unwrap();
        assert_eq!(payment.transactions_last5.len(), 0);
        assert_eq!(payment.block_heights.len(), 0);
        assert_eq!(payment.received_amount, ICPTs::from_icpts(0).unwrap());
        assert_eq!(payment.amount, ICPTs::from_icpts(10).unwrap());
        assert_eq!(payment.paid_at, None);
    }
}

mod cleanup_old_transactions {
    use super::*;

    const PRUNE_TRANSACTIONS_COUNT: u32 = 1000;

    #[test]
    fn clean_some_transactions() {
        let mut store = setup_test_store();
        let now = TimeStamp {
            timestamp_nanos: TEST_NOW_TIMESTAMP_NANOS + 1,
        };
        let request = AddPaymentRequest {
            amount: ICPTs::from_icpts(10).unwrap(),
            created_remark: "test remark".to_string(),
        };
        store.add_payment(request.clone(), get_allow_add_payment_principal(), now);

        let old_count = store.transactions.len();
        assert!(old_count > 0);
        let old_payments_count = store.payments.len();
        assert!(old_payments_count > 0);

        let count_to_removed = 3;
        // act
        store.cleanup_old_transactions(&now, count_to_removed);

        assert_eq!(
            store.transactions.len(),
            old_count - min(old_count, count_to_removed as usize)
        );
        assert_eq!(store.payments.len(), old_payments_count);
    }

    #[test]
    fn clean_some_payments() {
        let mut store = setup_test_store();
        let request = AddPaymentRequest {
            amount: ICPTs::from_icpts(10).unwrap(),
            created_remark: "test remark".to_string(),
        };
        store.add_payment(
            request.clone(),
            get_allow_add_payment_principal(),
            TimeStamp {
                timestamp_nanos: TEST_NOW_TIMESTAMP_NANOS,
            },
        );
        let now = TimeStamp {
            timestamp_nanos: TEST_NOW_TIMESTAMP_NANOS + MAX_PAYMENT_AGE_NANOS + 1,
        };
        let old_count = store.transactions.len();
        assert!(old_count > 0);
        let old_payments_count = store.payments.len();
        assert!(old_payments_count > 0);

        // act
        store.cleanup_old_transactions(&now, PRUNE_TRANSACTIONS_COUNT);

        assert_eq!(store.transactions.len(), 0);
        assert_eq!(store.payments.len(), 0);
    }
}

fn get_some_user_account_id() -> AccountIdentifier {
    let principal1 = PrincipalId::from_str(TEST_ACCOUNT_1).unwrap();
    let account_identifier1 = AccountIdentifier::from(principal1);
    account_identifier1
}

fn get_icnaming_ledger_account_id() -> AccountIdentifier {
    let principal2 = PrincipalId::from_str(TEST_ACCOUNT_2).unwrap();
    let account_identifier2 = AccountIdentifier::from(principal2.clone());
    account_identifier2
}

fn get_allow_add_payment_principal() -> PrincipalId {
    let principal3 = PrincipalId::from_str(TEST_ACCOUNT_3).unwrap();
    principal3
}

fn setup_test_store() -> PaymentsStore {
    let account_identifier1 = get_some_user_account_id();
    let icnaming_ledger_receiver_account_id = get_icnaming_ledger_account_id();

    STATE.with(|s| {
        let mut s = s.settings.borrow_mut();
        s.allow_caller_ids.insert(get_allow_add_payment_principal());
        s.receiver_icnaming_ledger_account_ids = hashset! {get_icnaming_ledger_account_id()};
    });

    let mut store = PaymentsStore::default();
    store.add_payment(
        AddPaymentRequest {
            amount: ICPTs::from_icpts(10).unwrap(),
            created_remark: "test remark".to_string(),
        },
        get_allow_add_payment_principal(),
        TimeStamp {
            timestamp_nanos: TEST_NOW_TIMESTAMP_NANOS,
        },
    );

    let timestamp = TimeStamp {
        timestamp_nanos: TEST_NOW_TIMESTAMP_NANOS,
    };
    {
        let transfer = Mint {
            amount: ICPTs::from_e8s(1_000_000_000),
            to: account_identifier1,
        };
        store
            .try_sync_transaction(transfer, Memo(1), 0, timestamp)
            .unwrap();
    }
    {
        let transfer = Mint {
            amount: ICPTs::from_e8s(1_000_000_000),
            to: account_identifier1,
        };
        store
            .try_sync_transaction(transfer, Memo(1), 1, timestamp)
            .unwrap();
    }
    {
        let transfer = Burn {
            amount: ICPTs::from_e8s(500_000_000),
            from: account_identifier1,
        };
        store
            .try_sync_transaction(transfer, Memo(1), 2, timestamp)
            .unwrap();
    }
    {
        let transfer = Send {
            amount: ICPTs::from_e8s(300_000_000),
            fee: ICPTs::from_e8s(1_000),
            from: account_identifier1,
            to: icnaming_ledger_receiver_account_id,
        };
        store
            .try_sync_transaction(transfer.clone(), Memo(1), 3, timestamp)
            .unwrap();
    }
    store
}

use candid::Principal;
use rstest::*;

use test_common::user::*;

use crate::user_quota_store::{QuotaType, UserQuotaStore};

#[fixture]
fn store() -> UserQuotaStore {
    UserQuotaStore::new()
}

#[rstest]
fn test_get_user_quota_not_set(store: UserQuotaStore, mock_user1: Principal) {
    let user_quota = store.get_quota(&mock_user1.clone(), &QuotaType::LenGte(4));
    assert_eq!(user_quota, None);
}

#[rstest]
fn test_add_quota(mut store: UserQuotaStore, mock_user1: Principal) {
    // create some quota types
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

    let len = quota_types.len();
    for i in 0..len {
        let quota_type = quota_types[i].clone();
        store.add_quota(mock_user1.clone(), quota_type.clone(), (i + 1) as u32);
        store.add_quota(mock_user1.clone(), quota_type.clone(), (i + 1) as u32);
    }

    // assert
    for i in 0..len {
        let quota_type = quota_types[i].clone();
        let user_quota = store.get_quota(&mock_user1.clone(), &quota_type);
        assert_eq!(user_quota, Some((i + 1) as u32 * 2));
    }
}

#[rstest]
fn test_sub_quota(mut store: UserQuotaStore, mock_user1: Principal) {
    store.add_quota(mock_user1.clone(), QuotaType::LenGte(4), 4);

    // act
    store.sub_quota(&mock_user1.clone(), &QuotaType::LenGte(4), 2);

    // assert
    let user_quota = store.get_quota(&mock_user1.clone(), &QuotaType::LenGte(4));
    assert_eq!(user_quota, Some(2));
}

mod transfer_quota {
    use super::*;

    #[rstest]
    fn test_transfer_quota_success(
        mut store: UserQuotaStore,
        mock_user1: Principal,
        mock_user2: Principal,
    ) {
        store.add_quota(mock_user1.clone(), QuotaType::LenGte(4), 4);

        // act
        let result = store.transfer_quota(&mock_user1, &mock_user2, &QuotaType::LenGte(4), 1);

        // assert
        assert!(result);
        let user_quota = store.get_quota(&mock_user1, &QuotaType::LenGte(4));
        assert_eq!(user_quota, Some(3));
        let user_quota = store.get_quota(&mock_user2, &QuotaType::LenGte(4));
        assert_eq!(user_quota, Some(1));
    }

    #[rstest]
    fn test_transfer_quota_fail_not_enough_quota(
        mut store: UserQuotaStore,
        mock_user1: Principal,
        mock_user2: Principal,
    ) {
        store.add_quota(mock_user1.clone(), QuotaType::LenGte(4), 4);

        // act
        let result = store.transfer_quota(&mock_user1, &mock_user2, &QuotaType::LenGte(4), 5);

        // assert
        assert!(!result);
        let user_quota = store.get_quota(&mock_user1, &QuotaType::LenGte(4));
        assert_eq!(user_quota, Some(4));
        let user_quota = store.get_quota(&mock_user2, &QuotaType::LenGte(4));
        assert_eq!(user_quota, None);
    }

    #[rstest]
    fn test_transfer_quota_fail_no_quota(
        mut store: UserQuotaStore,
        mock_user1: Principal,
        mock_user2: Principal,
    ) {
        // act
        let result = store.transfer_quota(&mock_user1, &mock_user2, &QuotaType::LenGte(4), 1);

        // assert
        assert!(!result);
        let user_quota = store.get_quota(&mock_user1, &QuotaType::LenGte(4));
        assert_eq!(user_quota, None);
        let user_quota = store.get_quota(&mock_user2, &QuotaType::LenGte(4));
        assert_eq!(user_quota, None);
    }
}

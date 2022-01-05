use candid::Principal;
use rstest::*;

use super::*;

#[fixture]
fn owner() -> Principal {
    Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap()
}

mod user_quota_manager {
    use super::*;

    #[fixture]
    fn user_quota_manager() -> UserQuotaManager {
        UserQuotaManager::new()
    }

    #[rstest]
    fn test_get_user_quota_not_set(user_quota_manager: UserQuotaManager, owner: Principal) {
        let user_quota = user_quota_manager.get_quota(&owner.clone(), &QuotaType::LenGte(4));
        assert_eq!(user_quota, None);
    }

    #[rstest]
    fn test_add_quota(mut user_quota_manager: UserQuotaManager, owner: Principal) {
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
            user_quota_manager.add_quota(owner.clone(), quota_type.clone(), i as u32);
            user_quota_manager.add_quota(owner.clone(), quota_type.clone(), i as u32);
        }

        // assert
        for i in 0..len {
            let quota_type = quota_types[i].clone();
            let user_quota = user_quota_manager.get_quota(&owner.clone(), &quota_type);
            assert_eq!(user_quota, Some(i as u32 * 2));
        }
    }

    #[rstest]
    fn test_sub_quota(mut user_quota_manager: UserQuotaManager, owner: Principal) {
        user_quota_manager.add_quota(owner.clone(), QuotaType::LenGte(4), 4);

        // act
        user_quota_manager.sub_quota(&owner.clone(), &QuotaType::LenGte(4), 2);

        // assert
        let user_quota = user_quota_manager.get_quota(&owner.clone(), &QuotaType::LenGte(4));
        assert_eq!(user_quota, Some(2));
    }
}

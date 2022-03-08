use std::borrow::Borrow;
use std::sync::Arc;

use candid::Principal;
use once_cell::sync::Lazy;
use rstest::*;

use common::cycles_minting_types::{IcpXdrConversionRate, IcpXdrConversionRateCertifiedResponse};
use common::icnaming_ledger_types::{AddPaymentResponse, Memo};
use test_common::canister_api::*;
use test_common::ic_api::init_test;
use test_common::user::*;

use super::*;

const TEST_QUOTA: QuotaType = QuotaType::LenGte(4);
const TEST_ADD_PAYMENT_RESPONSE: Lazy<AddPaymentResponse> = Lazy::new(|| AddPaymentResponse {
    payment_id: 23456,
    memo: Memo(669),
    payment_account_id: vec![
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
        26, 27, 28,
    ],
});

#[fixture]
fn owner() -> Principal {
    mock_user1()
}

#[fixture]
fn quota_owner() -> Principal {
    mock_user2()
}

#[fixture]
fn default_resolver() -> Principal {
    get_named_get_canister_id(CANISTER_NAME_RESOLVER)
}

#[fixture]
fn register_years() -> u32 {
    5
}

#[fixture]
fn service(
    _init_test: (),
    quota_owner: Principal,
    register_years: u32,
    mut mock_icnaming_ledger_api: MockICNamingLedgerApi,
    mut mock_cycles_minting_api: MockCyclesMintingApi,
    mut mock_registry_api: MockRegistryApi,
) -> RegistrarService {
    STATE.with(|s| {
        let mut m = s.user_quota_store.borrow_mut();
        m.add_quota(quota_owner, TEST_QUOTA, register_years);
    });
    let mut service = RegistrarService::new();
    mock_icnaming_ledger_api
        .expect_add_payment()
        .returning(|_| Ok(TEST_ADD_PAYMENT_RESPONSE.clone()));
    service.icnaming_ledger_api = Arc::new(mock_icnaming_ledger_api);
    mock_cycles_minting_api
        .expect_get_icp_xdr_conversion_rate()
        .returning(|| {
            Ok(IcpXdrConversionRateCertifiedResponse {
                certificate: Vec::new(),
                hash_tree: Vec::new(),
                data: IcpXdrConversionRate {
                    xdr_permyriad_per_icp: 20000u64,
                    timestamp_seconds: 1644303358u64,
                },
            })
        });
    service.cycles_minting_api = Arc::new(mock_cycles_minting_api);
    mock_registry_api
        .expect_reclaim_name()
        .returning(|name, owner, resolver| Ok(true));
    service.registry_api = Arc::new(mock_registry_api);
    service
}

fn assert_quota_count(quota_owner: &Principal, count: u32) {
    assert_quota_type_count(quota_owner, &TEST_QUOTA, count);
}

fn assert_quota_type_count(quota_owner: &Principal, quota_type: &QuotaType, count: u32) {
    STATE.with(|s| {
        let m = s.user_quota_store.borrow();
        assert_eq!(m.get_quota(quota_owner, quota_type).unwrap_or(0), count);
    });
}

mod normalized {
    use super::*;

    #[rstest]
    #[case("test", "test")]
    #[case("123", "123")]
    #[case(" trim_blank ", "trim_blank")]
    #[case(" TOLOWER ", "tolower")]
    #[case(" 你好 ", "你好")]
    fn test_normalized(service: RegistrarService, #[case] input: &str, #[case] expected: &str) {
        let normalized = service.normalize_name(input);
        assert_eq!(normalized, expected);
    }
}

mod validate_name {
    use super::*;

    #[rstest]
    #[case("nice.icp", Ok(NameParseResult::parse("nice.icp")))]
    #[case("ni-e.icp", Ok(NameParseResult::parse("ni-e.icp")))]
    #[case("n1-e.icp", Ok(NameParseResult::parse("n1-e.icp")))]
    #[case("www.nice.icp",
    Err("it must be second level name".to_string())
    )]
    #[case("nice.com",
    Err(format ! ("top level of name must be {}", TOP_LABEL))
    )]
    #[case("01234567890123456789012345678901234567890123456789012345678912345.icp",
    Err("second level name must be less than 64 characters".to_string())
    )]
    #[case("nic%.icp",
    Err("name must be alphanumeric or -".to_string()),
    )]
    #[case("你好.icp",
    Err("name must be alphanumeric or -".to_string()),
    )]
    #[case("n1-e .icp",
    Err("name must be alphanumeric or -".to_string()),
    )]
    fn test_validate_name(
        service: RegistrarService,
        #[case] input: &str,
        #[case] expected: Result<NameParseResult, String>,
    ) {
        let expected = expected.map_err(|e| ICNSError::InvalidName { reason: e });
        let result = service.validate_name(input);
        assert_eq!(result, expected);
    }
}

mod validate_quota {
    use super::*;

    #[rstest]
    #[case(NameParseResult::parse("nice.icp"),
    QuotaType::LenGte(3),
    Ok(()),
    )]
    #[case(NameParseResult::parse("nice.icp"),
    QuotaType::LenGte(4),
    Ok(()),
    )]
    #[case(NameParseResult::parse("nice.icp"),
    QuotaType::LenGte(5),
    Err("Name must be at least 5 characters long".to_string()),
    )]
    #[case(NameParseResult::parse("nice.icp"),
    QuotaType::LenEq(3),
    Err("Name must be exactly 3 characters long".to_string()),
    )]
    #[case(NameParseResult::parse("nice.icp"),
    QuotaType::LenEq(4),
    Ok(()),
    )]
    #[case(NameParseResult::parse("nice.icp"),
    QuotaType::LenEq(5),
    Err("Name must be exactly 5 characters long".to_string()),
    )]
    fn test_validate_quota(
        service: RegistrarService,
        owner: Principal,
        #[case] name: NameParseResult,
        #[case] quota_type: QuotaType,
        #[case] expected: Result<(), String>,
    ) {
        STATE.with(|s| {
            let mut m = s.user_quota_store.borrow_mut();
            m.add_quota(owner.clone(), quota_type.clone(), 1);
        });
        let result = service.validate_quota(&name, &owner, &quota_type, 1);
        assert_eq!(result, expected);
    }

    #[rstest]
    fn test_validate_quota_no_quota(service: RegistrarService, owner: Principal) {
        let name = NameParseResult::parse("nice.icp");
        let quota_type = QuotaType::LenGte(3);
        let result = service.validate_quota(&name, &owner, &quota_type, 1);
        assert_eq!(result, Err("User has no quota for len_gte(3)".to_string()));
    }

    #[rstest]
    fn test_validate_quota_not_enough_quota(service: RegistrarService, owner: Principal) {
        let quota_type = QuotaType::LenGte(3);
        STATE.with(|s| {
            let mut m = s.user_quota_store.borrow_mut();
            m.add_quota(owner.clone(), quota_type.clone(), 1);
        });
        let name = NameParseResult::parse("nice.icp");
        let result = service.validate_quota(&name, &owner, &quota_type, 2);
        assert_eq!(result, Err("User has no quota for len_gte(3)".to_string()));
    }
}

mod available {
    use super::*;

    #[rstest]
    fn test_available(service: RegistrarService) {
        {
            let result = service.available("www.nice.icp");
            assert_eq!(
                result,
                Err(ICNSError::InvalidName {
                    reason: "it must be second level name".to_string()
                })
            );
        }
        {
            let result = service.available("nice.icp");
            assert_eq!(result.is_ok(), true);
        }
        {
            let name = "nice.icp";
            STATE.with(|s| {
                let mut store = s.registration_store.borrow_mut();
                let registration =
                    Registration::new(Principal::anonymous(), name.to_string(), 0, 0);
                store.add_registration(registration);
            });
            let result = service.available(name);
            assert_eq!(result, Err(ICNSError::RegistrationHasBeenTaken));
        }
        {
            let name = "icnaming.icp";
            let result = service.available(name);
            assert_eq!(result, Err(ICNSError::RegistrationHasBeenTaken));
        }
    }
}

mod get_name_expires {
    use super::*;

    #[rstest]
    fn test_get_name_expires(service: RegistrarService) {
        {
            let name = "nice.icp";
            let expires = service.get_name_expires(name);
            assert_eq!(expires, Err(ICNSError::RegistrationNotFound));
        }
        {
            let name = "nice.icp";
            let expired_at = 123000000;
            STATE.with(|s| {
                let mut store = s.registration_store.borrow_mut();
                let registration =
                    Registration::new(Principal::anonymous(), name.to_string(), expired_at, 0);
                store.add_registration(registration);
            });
            let expires = service.get_name_expires(name);
            assert_eq!(expires, Ok(expired_at / 1000000));
        }
    }
}

mod get_names {
    use super::*;

    #[rstest]
    fn test_get_names_invalid_owner(service: RegistrarService) {
        let owner = Principal::anonymous();
        let input = GetPageInput {
            limit: 1,
            offset: 0,
        };
        let result = service.get_names(&owner, &input);
        assert!(result.is_err());
        match result {
            Err(ICNSError::Unauthorized) => {
                assert!(true);
            }
            _ => {
                assert!(false);
            }
        }
    }

    #[rstest]
    fn test_get_names_invalid_page(service: RegistrarService) {
        let owner = Principal::anonymous();
        let input = GetPageInput {
            limit: 0,
            offset: 0,
        };
        let result = service.get_names(&owner, &input);
        assert!(result.is_err());
        match result {
            Err(e) => {
                assert_eq!(
                    e,
                    ICNSError::ValueShouldBeInRangeError {
                        field: "limit".to_string(),
                        min: 1,
                        max: 100,
                    }
                )
            }
            _ => {
                assert!(false);
            }
        }
    }
}

mod register {
    use common::constants::{DEFAULT_MAX_REGISTRATION_YEAR, DEFAULT_MIN_REGISTRATION_YEAR};
    use common::dto::RegistryDto;

    use super::*;

    #[rstest]
    async fn test_register_err_name_invalid(
        mut service: RegistrarService,
        owner: Principal,
        register_years: u32,
        quota_owner: Principal,
    ) {
        let name = "www.nice.icp";
        let result = service
            .register(
                name,
                &owner,
                register_years,
                0,
                &quota_owner,
                TEST_QUOTA,
                false,
            )
            .await;
        assert_quota_count(&quota_owner, register_years);
        assert_eq!(
            result,
            Err(ICNSError::InvalidName {
                reason: "it must be second level name".to_string()
            })
        );
    }

    #[rstest]
    async fn test_register_err_owner_invalid(
        mut service: RegistrarService,
        quota_owner: Principal,
        register_years: u32,
    ) {
        let owner = Principal::anonymous();
        let name = "nice.icp";
        let result = service
            .register(
                name,
                &owner,
                register_years,
                0,
                &quota_owner,
                TEST_QUOTA,
                false,
            )
            .await;
        assert_quota_count(&quota_owner, register_years);
        assert_eq!(result, Err(ICNSError::Unauthorized));
    }

    #[rstest]
    async fn test_register_err_lack_of_quota(
        mut service: RegistrarService,
        owner: Principal,
        quota_owner: Principal,
        register_years: u32,
    ) {
        let name = "nice.icp";
        STATE.with(|s| {
            let mut quota_manager = s.user_quota_store.borrow_mut();
            quota_manager.sub_quota(&quota_owner.to_owned(), &TEST_QUOTA, register_years - 1);
        });

        // act
        let result = service
            .register(
                name,
                &owner,
                register_years,
                0,
                &quota_owner,
                TEST_QUOTA,
                false,
            )
            .await;
        assert_quota_count(&quota_owner, 1);
        assert_eq!(
            result,
            Err(ICNSError::InvalidName {
                reason: "User has no quota for len_gte(4)".to_string()
            })
        );
    }

    #[rstest]
    async fn test_register_err_year_invalid(
        mut service: RegistrarService,
        owner: Principal,
        quota_owner: Principal,
        register_years: u32,
    ) {
        let name = "nice.icp";
        let result = service
            .register(name, &owner, 15, 0, &quota_owner, TEST_QUOTA, false)
            .await;
        assert_quota_count(&quota_owner, register_years);
        assert_eq!(
            result,
            Err(ICNSError::YearsRangeError {
                min: DEFAULT_MIN_REGISTRATION_YEAR,
                max: DEFAULT_MAX_REGISTRATION_YEAR,
            })
        );
    }

    #[rstest]
    async fn test_register_err_already_taken(
        mut service: RegistrarService,
        owner: Principal,
        quota_owner: Principal,
        register_years: u32,
    ) {
        let name = "nice.icp";
        STATE.with(|s| {
            let mut store = s.registration_store.borrow_mut();
            let registration = Registration::new(owner, name.to_string(), 0, 0);
            store.add_registration(registration);
        });
        let result = service
            .register(
                name,
                &owner,
                register_years,
                0,
                &quota_owner,
                TEST_QUOTA,
                false,
            )
            .await;
        assert_quota_count(&quota_owner, register_years);
        assert_eq!(result, Err(ICNSError::RegistrationHasBeenTaken));
    }

    #[rstest]
    async fn test_register_err_reserved(
        mut service: RegistrarService,
        owner: Principal,
        quota_owner: Principal,
        register_years: u32,
    ) {
        let name = "icnaming.icp";
        let result = service
            .register(
                name,
                &owner,
                register_years,
                0,
                &quota_owner,
                TEST_QUOTA,
                false,
            )
            .await;
        assert_quota_count(&quota_owner, register_years);
        assert_eq!(result, Err(ICNSError::RegistrationHasBeenTaken));
    }

    #[rstest]
    async fn test_register_api_failed(
        mut service: RegistrarService,
        owner: Principal,
        quota_owner: Principal,
        default_resolver: Principal,
        register_years: u32,
        mut mock_registry_api: MockRegistryApi,
    ) {
        let name = "nice.icp";
        let now = 0;

        let _ctx = mock_registry_api.expect_set_subdomain_owner().returning(
            move |label, parent_name, sub_owner, ttl, resolver| {
                assert_eq!(label, "nice");
                assert_eq!(parent_name, TOP_LABEL.to_string());
                assert_eq!(sub_owner, owner);
                assert_eq!(ttl, DEFAULT_TTL);
                assert_eq!(resolver, default_resolver);
                Err(ICNSError::Unknown.into())
            },
        );
        service.registry_api = Arc::new(mock_registry_api);

        // act
        let result = service
            .register(
                name,
                &owner,
                register_years,
                now,
                &quota_owner,
                TEST_QUOTA,
                false,
            )
            .await;

        // assert
        assert_quota_count(&quota_owner, register_years);
        STATE.with(|s| {
            let store = s.registration_store.borrow();
            assert_eq!(store.get_registrations().borrow().len(), 0);
        });
        assert_eq!(
            result,
            Err(ICNSError::RemoteError(ICNSError::Unknown.into()))
        );
    }

    #[rstest]
    async fn test_register_success(
        mut service: RegistrarService,
        owner: Principal,
        quota_owner: Principal,
        default_resolver: Principal,
        register_years: u32,
        mut mock_registry_api: MockRegistryApi,
    ) {
        let name = "nice.icp";
        let now = 0;

        let _ctx = mock_registry_api.expect_set_subdomain_owner().returning(
            move |label, parent_name, sub_owner, ttl, resolver| {
                assert_eq!(label, "nice");
                assert_eq!(parent_name, TOP_LABEL.to_string());
                assert_eq!(sub_owner, owner);
                assert_eq!(ttl, DEFAULT_TTL);
                assert_eq!(resolver, default_resolver);
                Ok(RegistryDto {
                    owner,
                    name: name.to_string(),
                    ttl,
                    resolver,
                })
            },
        );
        service.registry_api = Arc::new(mock_registry_api);

        // act
        let result = service
            .register(
                name,
                &owner,
                register_years,
                now,
                &quota_owner,
                TEST_QUOTA,
                false,
            )
            .await;

        // assert
        assert_quota_count(&quota_owner, 0);
        STATE.with(|s| {
            let store = s.registration_store.borrow();
            let registrations = store.get_registrations();
            assert_eq!(
                registrations.borrow().get(&name.to_string()),
                Some(&Registration::new(
                    owner,
                    name.to_string(),
                    now + year_to_ns(register_years),
                    now,
                ))
            );
        });
        assert_eq!(result, Ok(true));
    }
}

mod validate_quota_order_details {
    use super::*;

    #[rstest]
    fn test_validate_quota_order_details_ok(mock_user1: Principal) {
        let mut details = HashMap::new();
        let mut items = HashMap::new();
        items.insert(QuotaType::LenEq(7), 2);
        items.insert(QuotaType::LenGte(6), 3);
        details.insert(mock_user1, items);

        // act
        let result = validate_quota_order_details(&details);

        // assert
        assert_eq!(result, Ok(()));
    }

    #[rstest]
    fn test_validate_quota_order_details_anonymous() {
        let mut details = HashMap::new();
        let mut items = HashMap::new();
        items.insert(QuotaType::LenEq(7), 2);
        items.insert(QuotaType::LenGte(6), 0);
        details.insert(Principal::anonymous(), items);
        // act
        let result = validate_quota_order_details(&details);

        // assert
        assert_eq!(result, Err(ICNSError::Unauthorized));
    }

    #[rstest]
    fn test_validate_quota_order_details_empty_items(_mock_user1: Principal) {
        let details = HashMap::new();

        // act
        let result = validate_quota_order_details(&details);

        // assert
        assert_eq!(result, Err(ICNSError::InvalidQuotaOrderDetails));
    }

    #[rstest]
    fn test_validate_quota_order_details_amount_0(mock_user1: Principal) {
        let mut details = HashMap::new();
        let mut items = HashMap::new();
        items.insert(QuotaType::LenEq(7), 2);
        items.insert(QuotaType::LenGte(6), 0);
        details.insert(mock_user1, items);

        // act
        let result = validate_quota_order_details(&details);

        // assert
        assert_eq!(result, Err(ICNSError::InvalidQuotaOrderDetails));
    }

    #[rstest]
    fn test_validate_quota_order_details_too_much_amount(mock_user1: Principal) {
        let mut details = HashMap::new();
        let mut items = HashMap::new();
        items.insert(QuotaType::LenEq(7), 2);
        items.insert(QuotaType::LenGte(6), MAX_QUOTA_ORDER_AMOUNT_EACH_TYPE + 1);
        details.insert(mock_user1, items);

        // act
        let result = validate_quota_order_details(&details);

        // assert
        assert_eq!(result, Err(ICNSError::InvalidQuotaOrderDetails));
    }
}

mod get_quota_type_price {
    use super::*;

    #[rstest]
    #[case(6, 110_000_000u64)]
    #[case(7, 100_000_000u64)]
    #[case(8, 100_000_000u64)]
    fn test_get_quota_type_price(#[case] len: u8, #[case] amount: u64) {
        let xdr_permyriad_per_icp = 20000;
        let result = get_quota_type_price_in_icp_e8s(&QuotaType::LenEq(len), xdr_permyriad_per_icp);
        assert_eq!(result, amount);
        let result =
            get_quota_type_price_in_icp_e8s(&QuotaType::LenGte(len), xdr_permyriad_per_icp);
        assert_eq!(result, amount);
    }
}

mod get_price_for_quota_order_details_in_icp_e8s {
    use super::*;

    #[rstest]
    fn test_get_price_for_quota_order_details_in_icp_e8s(mock_user1: Principal) {
        let xdr_permyriad_per_icp = 20000;
        let mut details = HashMap::new();
        let mut items = HashMap::new();
        items.insert(QuotaType::LenEq(7), 2);
        items.insert(QuotaType::LenGte(6), 3);
        details.insert(mock_user1, items);

        let result = get_price_for_quota_order_details_in_icp_e8s(&details, xdr_permyriad_per_icp);

        assert_eq!(result, 530_000_000u64);
    }
}

mod apply_quota_order_details {
    use super::*;

    #[rstest]
    fn test_apply_quota_order_details_ok(mock_user1: Principal, mock_user2: Principal) {
        let mut details = HashMap::new();
        let mut items = HashMap::new();
        items.insert(QuotaType::LenEq(1), 2);
        items.insert(QuotaType::LenGte(2), 3);
        details.insert(mock_user1, items);
        let mut items = HashMap::new();
        items.insert(QuotaType::LenEq(3), 4);
        items.insert(QuotaType::LenGte(4), 5);
        details.insert(mock_user2, items);

        // act
        apply_quota_order_details(&details);

        // assert
        STATE.with(|s| {
            let user_quota_manager = s.user_quota_store.borrow();
            assert_eq!(
                user_quota_manager.get_quota(&mock_user1, &QuotaType::LenEq(1)),
                Some(2)
            );
            assert_eq!(
                user_quota_manager.get_quota(&mock_user1, &QuotaType::LenGte(2)),
                Some(3)
            );
            assert_eq!(
                user_quota_manager.get_quota(&mock_user2, &QuotaType::LenEq(3)),
                Some(4)
            );
            assert_eq!(
                user_quota_manager.get_quota(&mock_user2, &QuotaType::LenGte(4)),
                Some(5)
            );
        });
    }
}

mod quota_order_manager {
    use super::*;

    #[fixture]
    fn mock_details(mock_user1: Principal, mock_user2: Principal) -> QuotaOrderDetails {
        let mut details = HashMap::new();
        let mut items = HashMap::new();
        items.insert(QuotaType::LenEq(7), 2);
        items.insert(QuotaType::LenGte(6), 3);
        details.insert(mock_user1, items);
        let mut items = HashMap::new();
        items.insert(QuotaType::LenEq(7), 10);
        items.insert(QuotaType::LenGte(6), 9);
        details.insert(mock_user2, items);
        details
    }

    mod get_order {
        use super::*;

        #[rstest]
        async fn test_get_order_ok(
            service: RegistrarService,
            mock_user1: Principal,
            mock_now: u64,
            mock_details: QuotaOrderDetails,
        ) {
            service
                .place_quota_order(&mock_user1, mock_now, mock_details.clone())
                .await;

            // act
            let order = service.get_quota_order(&mock_user1).unwrap().unwrap();

            // assert
            assert_eq!(order.id, 1);
            assert_eq!(order.created_user, mock_user1);
            assert_eq!(order.details, mock_details);
            assert_eq!(order.created_at, mock_now);
            assert_eq!(order.status, QuotaOrderStatus::New);
            let payment = QuotaOrderPayment::new(
                TEST_ADD_PAYMENT_RESPONSE.payment_id,
                PaymentType::ICP,
                Nat(BigUint::from(2520_000_000u64)),
                PaymentMemo::ICP(ICPMemo(TEST_ADD_PAYMENT_RESPONSE.memo.0)),
                TEST_ADD_PAYMENT_RESPONSE.payment_account_id.clone(),
            );
            assert_eq!(order.payment, payment);
            assert_eq!(order.paid_at, None);
            assert_eq!(order.canceled_at, None);
        }

        #[rstest]
        fn test_get_order_none(
            service: RegistrarService,
            mock_user1: Principal,
            _mock_now: u64,
            _mock_details: QuotaOrderDetails,
        ) {
            // act
            let order = service.get_quota_order(&mock_user1).unwrap();

            // assert
            assert_eq!(order, None);
        }
    }

    mod place_order {
        use super::*;

        #[rstest]
        async fn test_place_order_ok(
            service: RegistrarService,
            mock_user1: Principal,
            mock_now: u64,
            mock_details: QuotaOrderDetails,
        ) {
            // act
            let result = service
                .place_quota_order(&mock_user1, mock_now, mock_details)
                .await;

            // assert
            let order = service.get_quota_order(&mock_user1).unwrap().unwrap();
            assert_eq!(result, Ok(PlaceOrderOutput { order }));
        }

        #[rstest]
        async fn test_place_order_already_placed(
            service: RegistrarService,
            mock_user1: Principal,
            mock_now: u64,
            mock_details: QuotaOrderDetails,
        ) {
            // arrange
            service
                .place_quota_order(&mock_user1, mock_now, mock_details.clone())
                .await
                .unwrap();

            // act
            let result = service
                .place_quota_order(&mock_user1, mock_now, mock_details)
                .await;

            // assert
            assert_eq!(result, Err(ICNSError::PendingOrder));
        }
    }

    mod cancel_order {
        use super::*;

        #[rstest]
        async fn test_cancel_order_ok(
            service: RegistrarService,
            mock_user1: Principal,
            mock_now: u64,
            mock_details: QuotaOrderDetails,
        ) {
            service
                .place_quota_order(&mock_user1, mock_now, mock_details)
                .await;
            // act
            service.cancel_quota_order(&mock_user1, mock_now).unwrap();

            // assert
            let order = service.get_quota_order(&mock_user1).unwrap();
            assert_eq!(order, None);
        }

        #[rstest]
        fn test_cancel_order_none(service: RegistrarService, mock_user1: Principal, mock_now: u64) {
            // act
            let result = service.cancel_quota_order(&mock_user1, mock_now);

            // assert
            assert_eq!(result, Err(ICNSError::OrderNotFound));
        }
    }

    mod paid_order {
        use super::*;

        #[rstest]
        async fn test_paid_order_ok(
            service: RegistrarService,
            mock_user1: Principal,
            mock_now: u64,
            mock_details: QuotaOrderDetails,
        ) {
            let _ = service
                .place_quota_order(&mock_user1, mock_now, mock_details.clone())
                .await;
            // act
            let _result = service.paid_quota_order(TEST_ADD_PAYMENT_RESPONSE.payment_id, mock_now);

            // assert
            assert_eq!(
                service.get_quota_order(&mock_user1).unwrap().is_none(),
                true
            );

            STATE.with(|s| {
                let uqm = s.user_quota_store.borrow();
                for (user, quotas) in mock_details {
                    for (t, value) in quotas.iter() {
                        assert_eq!(uqm.get_quota(&user, t).unwrap(), *value);
                    }
                }
            });
        }
    }
}

mod get_price_in_icp_e8s {
    use super::*;

    #[rstest]
    #[case(7, 20000, 100_000_000)]
    #[case(7, 30000, 66_660_000)]
    #[case(3, 174_132, 16_760_000)]
    fn test_get_price_in_icp_e8s(
        #[case] len: u8,
        #[case] xdr_permyriad_per_icp: u64,
        #[case] expected: u64,
    ) {
        // act
        let result = get_price_in_icp_e8s(len, xdr_permyriad_per_icp);

        // assert
        assert_eq!(result, expected);
    }
}

mod cancel_expired_orders {
    use super::*;

    #[rstest]
    async fn test_cancel_expired_orders_clean_expired_success(
        service: RegistrarService,
        mock_now: u64,
        mock_user1: Principal,
        mock_user2: Principal,
    ) {
        service.submit_order(
            &mock_user1,
            mock_now,
            SubmitOrderRequest {
                name: "test-name.icp".to_string(),
                years: 1,
            },
        ).await;

        service.submit_order(
            &mock_user2,
            mock_now - EXPIRE_TIME_OF_NAME_ORDER_IN_NS - 1,
            SubmitOrderRequest {
                name: "test-name2.icp".to_string(),
                years: 1,
            },
        ).await;

        // act
        service.cancel_expired_orders(mock_now).unwrap();

        // assert
        STATE.with(|s| {
            let store = s.name_order_store.borrow();
            assert_eq!(store.get_order(&mock_user1).is_some(), true);
            assert_eq!(store.get_order(&mock_user2).is_none(), true);
        });
    }

    #[rstest]
    async fn test_cancel_expired_orders_availability_check_name_has_been_taken(
        service: RegistrarService,
        mock_now: u64,
        mock_user1: Principal,
        mock_user2: Principal,
    ) {
        service.submit_order(
            &mock_user1,
            mock_now,
            SubmitOrderRequest {
                name: "test-name.icp".to_string(),
                years: 1,
            },
        ).await;

        service.submit_order(
            &mock_user2,
            mock_now - EXPIRE_TIME_OF_NAME_ORDER_AVAILABILITY_CHECK_IN_NS + 1,
            SubmitOrderRequest {
                name: "test-name2.icp".to_string(),
                years: 1,
            },
        ).await;

        STATE.with(|s| {
            let mut store = s.registration_store.borrow_mut();
            store.add_registration(Registration::new(
                mock_user2.clone(),
                "test-name2.icp".to_string(),
                mock_now + 1111,
                mock_now,
            ));
        });

        // act
        service.cancel_expired_orders(mock_now).unwrap();

        // assert
        STATE.with(|s| {
            let store = s.name_order_store.borrow();
            assert_eq!(store.get_order(&mock_user1).is_some(), true);
            assert_eq!(store.get_order(&mock_user2).is_none(), true);
        });
    }


    #[rstest]
    async fn test_cancel_expired_orders_availability_check_name_is_not_taken(
        service: RegistrarService,
        mock_now: u64,
        mock_user1: Principal,
        mock_user2: Principal,
    ) {
        service.submit_order(
            &mock_user1,
            mock_now,
            SubmitOrderRequest {
                name: "test-name.icp".to_string(),
                years: 1,
            },
        ).await;

        service.submit_order(
            &mock_user2,
            mock_now - EXPIRE_TIME_OF_NAME_ORDER_AVAILABILITY_CHECK_IN_NS + 1,
            SubmitOrderRequest {
                name: "test-name2.icp".to_string(),
                years: 1,
            },
        ).await;

        // act
        service.cancel_expired_orders(mock_now).unwrap();

        // assert
        STATE.with(|s| {
            let store = s.name_order_store.borrow();
            assert_eq!(store.get_order(&mock_user1).is_some(), true);
            assert_eq!(store.get_order(&mock_user2).is_some(), true);
        });
    }
}

mod reclaim_name {
    use common::permissions::get_admin;
    use super::*;

    #[rstest]
    async fn reclaim_name_success(service: RegistrarService,
                                  mock_now: u64,
                                  mock_user1: Principal,
                                  mock_user2: Principal, ) {
        STATE.with(|s| {
            let mut store = s.registration_store.borrow_mut();
            store.add_registration(Registration::new(
                mock_user1.clone(),
                "test-name.icp".to_string(),
                mock_now + 1111,
                mock_now,
            ));
        });

        // act
        let reclaim_result = service.reclaim_name(
            "test-name.icp",
            &mock_user1).await;

        assert_eq!(reclaim_result.is_ok(), true);
    }

    #[rstest]
    async fn reclaim_name_success_admin_request(service: RegistrarService,
                                                mock_now: u64,
                                                mock_user1: Principal,
                                                mock_user2: Principal, ) {
        STATE.with(|s| {
            let mut store = s.registration_store.borrow_mut();
            store.add_registration(Registration::new(
                mock_user1.clone(),
                "test-name.icp".to_string(),
                mock_now + 1111,
                mock_now,
            ));
        });

        // act
        let reclaim_result = service.reclaim_name(
            "test-name.icp",
            &get_admin()).await;

        assert_eq!(reclaim_result.is_ok(), true);
    }

    #[rstest]
    async fn reclaim_name_failed_name_not_found(service: RegistrarService,
                                                mock_user1: Principal, ) {

        // act
        let reclaim_result = service.reclaim_name(
            "test-name.icp",
            &mock_user1).await;

        assert_eq!(reclaim_result.err().unwrap(), ICNSError::RegistrationNotFound);
    }


    #[rstest]
    async fn reclaim_name_failed_caller_error(service: RegistrarService,
                                              mock_now: u64,
                                              mock_user1: Principal,
                                              mock_user2: Principal, ) {
        STATE.with(|s| {
            let mut store = s.registration_store.borrow_mut();
            store.add_registration(Registration::new(
                mock_user1.clone(),
                "test-name.icp".to_string(),
                mock_now + 1111,
                mock_now,
            ));
        });

        // act
        let reclaim_result = service.reclaim_name(
            "test-name.icp",
            &mock_user2).await;

        // assert
        assert_eq!(reclaim_result.err().unwrap(), ICNSError::PermissionDenied);
    }
}
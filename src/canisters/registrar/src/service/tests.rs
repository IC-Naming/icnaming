use std::borrow::Borrow;
use std::sync::Arc;

use candid::Principal;

use rstest::*;

use common::cycles_minting_types::{IcpXdrConversionRate, IcpXdrConversionRateCertifiedResponse};
use test_common::canister_api::*;
use test_common::ic_api::init_test;
use test_common::user::*;

use super::*;

const TEST_QUOTA: QuotaType = QuotaType::LenGte(4);

#[fixture]
fn owner() -> AuthPrincipal {
    AuthPrincipal(mock_user1())
}

#[fixture]
fn quota_owner() -> AuthPrincipal {
    AuthPrincipal(mock_user2())
}

#[fixture]
fn default_resolver() -> Principal {
    get_named_get_canister_id(CanisterNames::Resolver)
}

#[fixture]
fn register_years() -> u32 {
    5
}

fn create_test_name(name: &str) -> String {
    format!("{}.{}", name, NAMING_TOP_LABEL)
}

#[fixture]
fn service(
    _init_test: (),
    quota_owner: AuthPrincipal,
    register_years: u32,
    mut mock_cycles_minting_api: MockCyclesMintingApi,
    mut mock_registry_api: MockRegistryApi,
    _mock_dicp_api: MockDICPApi,
    mut mock_resolver_api: MockResolverApi,
) -> RegistrarService {
    STATE.with(|s| {
        let mut m = s.user_quota_store.borrow_mut();
        m.add_quota(quota_owner, TEST_QUOTA, register_years);
    });
    let mut service = RegistrarService::default();
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
        .returning(|_name, _owner, _resolver| Ok(true));
    mock_resolver_api
        .expect_set_record_value()
        .returning(|_name: String, _patch_values: HashMap<String, String>| Ok(true));

    service.registry_api = Arc::new(mock_registry_api);
    service.resolver_api = Arc::new(mock_resolver_api);
    service
}

fn assert_quota_count(quota_owner: &AuthPrincipal, count: u32) {
    assert_quota_type_count(quota_owner, &TEST_QUOTA, count);
}

fn assert_quota_type_count(quota_owner: &AuthPrincipal, quota_type: &QuotaType, count: u32) {
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
    fn test_normalized(#[case] input: &str, #[case] expected: &str) {
        let normalized = normalize_name(input);
        assert_eq!(normalized.0.as_str(), expected);
    }
}

mod validate_name {
    use super::*;

    #[rstest]
    #[case(
        create_test_name("nice"),
        Ok(FirstLevelName::from(create_test_name("nice")))
    )]
    #[case(
        create_test_name("ni-e"),
        Ok(FirstLevelName::from(create_test_name("ni-e")))
    )]
    #[case(
        create_test_name("n1-e"),
        Ok(FirstLevelName::from(create_test_name("n1-e")))
    )]
    #[case(create_test_name("www.nice"),
    Err("it must be second level name".to_string())
    )]
    #[case("nice.com",
    Err(format ! ("top level of name must be {}", NAMING_TOP_LABEL))
    )]
    #[case(create_test_name("01234567890123456789012345678901234567890123456789012345678912345"),
    Err("second level name must be less than 64 characters".to_string())
    )]
    #[case(create_test_name("nic%"),
    Err("name must be alphanumeric or -".to_string()),
    )]
    #[case(create_test_name("你好"),
    Err("name must be alphanumeric or -".to_string()),
    )]
    #[case(create_test_name("n1-e "),
    Err("name must be alphanumeric or -".to_string()),
    )]
    fn test_validate_name(#[case] input: String, #[case] expected: Result<FirstLevelName, String>) {
        let expected = expected.map_err(|e| NamingError::InvalidName { reason: e });
        let result = validate_name(input.as_str());
        assert_eq!(result, expected);
    }
}

mod validate_quota {
    use super::*;

    #[rstest]
    #[case(create_test_name("nice"),
    QuotaType::LenGte(3),
    Ok(()),
    )]
    #[case(create_test_name("nice"),
    QuotaType::LenGte(4),
    Ok(()),
    )]
    #[case(create_test_name("nice"),
    QuotaType::LenGte(5),
    Err("Name must be at least 5 characters long".to_string()),
    )]
    #[case(create_test_name("nice"),
    QuotaType::LenEq(3),
    Err("Name must be exactly 3 characters long".to_string()),
    )]
    #[case(create_test_name("nice"),
    QuotaType::LenEq(4),
    Ok(()),
    )]
    #[case(create_test_name("nice"),
    QuotaType::LenEq(5),
    Err("Name must be exactly 5 characters long".to_string()),
    )]
    fn test_validate_quota(
        service: RegistrarService,
        owner: AuthPrincipal,
        #[case] name: String,
        #[case] quota_type: QuotaType,
        #[case] expected: Result<(), String>,
    ) {
        STATE.with(|s| {
            let mut m = s.user_quota_store.borrow_mut();
            m.add_quota(owner.clone(), quota_type.clone(), 1);
        });
        let name = FirstLevelName::from(name.as_str());
        let result = service.validate_quota(&name, &owner, &quota_type, 1);
        assert_eq!(result, expected);
    }

    #[rstest]
    fn test_validate_quota_no_quota(service: RegistrarService, owner: AuthPrincipal) {
        let name = FirstLevelName::from(create_test_name("nice"));
        let quota_type = QuotaType::LenGte(3);
        let result = service.validate_quota(&name, &owner, &quota_type, 1);
        assert_eq!(result, Err("User has no quota for len_gte(3)".to_string()));
    }

    #[rstest]
    fn test_validate_quota_not_enough_quota(service: RegistrarService, owner: AuthPrincipal) {
        let quota_type = QuotaType::LenGte(3);
        STATE.with(|s| {
            let mut m = s.user_quota_store.borrow_mut();
            m.add_quota(owner.clone(), quota_type.clone(), 1);
        });
        let name = FirstLevelName::from(create_test_name("nice"));
        let result = service.validate_quota(&name, &owner, &quota_type, 2);
        assert_eq!(result, Err("User has no quota for len_gte(3)".to_string()));
    }
}

mod available {
    use super::*;

    #[rstest]
    fn test_available(service: RegistrarService) {
        {
            let result = service.available(create_test_name("www.nice").as_str());
            assert_eq!(
                result,
                Err(NamingError::InvalidName {
                    reason: "it must be second level name".to_string()
                })
            );
        }
        {
            let result = service.available(create_test_name("nice").as_str());
            assert_eq!(result.is_ok(), true);
        }
        {
            let name = create_test_name("nice");
            STATE.with(|s| {
                let mut store = s.registration_store.borrow_mut();
                let registration =
                    Registration::new(Principal::anonymous(), name.to_string(), 0, 0);
                store.add_registration(registration);
            });
            let result = service.available(name.as_str());
            assert_eq!(result, Err(NamingError::RegistrationHasBeenTaken));
        }
        {
            let name = create_test_name("icnaming");
            let result = service.available(name.as_str());
            assert_eq!(result, Err(NamingError::RegistrationHasBeenTaken));
        }
    }
}

mod get_name_status {
    use super::*;

    #[rstest]
    fn test_get_name_status_available(service: RegistrarService) {
        let result = service
            .get_name_status(create_test_name("nice").as_str())
            .unwrap();
        assert_eq!(result.available, true);
        assert_eq!(result.registered, false);
        assert_eq!(result.kept, false);
        assert_eq!(result.details, None);
    }

    #[rstest]
    fn test_get_name_status_reserved(service: RegistrarService) {
        let result = service
            .get_name_status(create_test_name("icnaming").as_str())
            .unwrap();
        assert_eq!(result.available, false);
        assert_eq!(result.registered, false);
        assert_eq!(result.kept, true);
        assert_eq!(result.details, None);
    }

    #[rstest]
    fn test_get_name_status_registered(service: RegistrarService) {
        let name = create_test_name("nice");
        let registration = Registration::new(Principal::anonymous(), name.to_string(), 0, 0);
        STATE.with(|s| {
            let mut store = s.registration_store.borrow_mut();
            store.add_registration(registration.clone());
        });
        let result = service.get_name_status(name.as_str()).unwrap();
        assert_eq!(result.available, false);
        assert_eq!(result.registered, true);
        assert_eq!(result.kept, false);
        assert_eq!(
            result.details.unwrap(),
            RegistrationDetails::from(&registration)
        );
    }
}

mod get_name_expires {
    use super::*;

    #[rstest]
    fn test_get_name_expires(service: RegistrarService) {
        {
            let name = create_test_name("nice");
            let expires = service.get_name_expires(name.as_str());
            assert_eq!(expires, Err(NamingError::RegistrationNotFound));
        }
        {
            let name = create_test_name("nice");
            let expired_at = 123000000;
            STATE.with(|s| {
                let mut store = s.registration_store.borrow_mut();
                let registration =
                    Registration::new(Principal::anonymous(), name.to_string(), expired_at, 0);
                store.add_registration(registration);
            });
            let expires = service.get_name_expires(name.as_str());
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
            Err(NamingError::Unauthorized) => {
                assert!(true);
            }
            _ => {
                assert!(false);
            }
        }
    }
}

mod register {
    use common::constants::{NAMING_MAX_REGISTRATION_YEAR, NAMING_MIN_REGISTRATION_YEAR};
    use common::dto::RegistryDto;

    use super::*;

    #[rstest]
    async fn test_register_err_name_invalid(
        mut service: RegistrarService,
        owner: AuthPrincipal,
        register_years: u32,
        quota_owner: AuthPrincipal,
        mock_now: u64,
    ) {
        let name = create_test_name("www.nice");
        let context = RegisterCoreContext::new(
            name.to_string(),
            owner,
            register_years,
            TimeInNs(mock_now),
            false,
        );
        let result = service
            .register_with_quota_core(context, &quota_owner, TEST_QUOTA)
            .await;
        assert_quota_count(&quota_owner, register_years);
        assert_eq!(
            result,
            Err(NamingError::InvalidName {
                reason: "it must be second level name".to_string()
            })
        );
    }

    #[rstest]
    async fn test_register_err_lack_of_quota(
        mut service: RegistrarService,
        owner: AuthPrincipal,
        quota_owner: AuthPrincipal,
        register_years: u32,
        mock_now: u64,
    ) {
        let name = create_test_name("nice");
        STATE.with(|s| {
            let mut quota_manager = s.user_quota_store.borrow_mut();
            quota_manager.sub_quota(&quota_owner.to_owned(), &TEST_QUOTA, register_years - 1);
        });

        // act
        let context = RegisterCoreContext::new(
            name.to_string(),
            owner,
            register_years,
            TimeInNs(mock_now),
            false,
        );
        let result = service
            .register_with_quota_core(context, &quota_owner, TEST_QUOTA)
            .await;
        assert_quota_count(&quota_owner, 1);
        assert_eq!(
            result,
            Err(NamingError::InvalidName {
                reason: "User has no quota for len_gte(4)".to_string()
            })
        );
    }

    #[rstest]
    async fn test_register_err_year_invalid(
        mut service: RegistrarService,
        owner: AuthPrincipal,
        quota_owner: AuthPrincipal,
        register_years: u32,
        mock_now: u64,
    ) {
        let name = create_test_name("nice");
        let context =
            RegisterCoreContext::new(name.to_string(), owner, 15, TimeInNs(mock_now), false);
        let result = service
            .register_with_quota_core(context, &quota_owner, TEST_QUOTA)
            .await;
        assert_quota_count(&quota_owner, register_years);
        assert_eq!(
            result,
            Err(NamingError::YearsRangeError {
                min: NAMING_MIN_REGISTRATION_YEAR,
                max: NAMING_MAX_REGISTRATION_YEAR,
            })
        );
    }

    #[rstest]
    async fn test_register_err_already_taken(
        mut service: RegistrarService,
        owner: AuthPrincipal,
        quota_owner: AuthPrincipal,
        register_years: u32,
        mock_now: u64,
    ) {
        let name = create_test_name("nice");
        STATE.with(|s| {
            let mut store = s.registration_store.borrow_mut();
            let registration = Registration::new(owner.0, name.to_string(), 0, 0);
            store.add_registration(registration);
        });
        let context = RegisterCoreContext::new(
            name.to_string(),
            owner,
            register_years,
            TimeInNs(mock_now),
            false,
        );
        let result = service
            .register_with_quota_core(context, &quota_owner, TEST_QUOTA)
            .await;
        assert_quota_count(&quota_owner, register_years);
        assert_eq!(result, Err(NamingError::RegistrationHasBeenTaken));
    }

    #[rstest]
    async fn test_register_err_reserved(
        mut service: RegistrarService,
        owner: AuthPrincipal,
        quota_owner: AuthPrincipal,
        register_years: u32,
        mock_now: u64,
    ) {
        let name = create_test_name("icnaming");
        let context = RegisterCoreContext::new(
            name.to_string(),
            owner,
            register_years,
            TimeInNs(mock_now),
            false,
        );
        let result = service
            .register_with_quota_core(context, &quota_owner, TEST_QUOTA)
            .await;
        assert_quota_count(&quota_owner, register_years);
        assert_eq!(result, Err(NamingError::RegistrationHasBeenTaken));
    }

    #[rstest]
    async fn test_register_api_failed(
        mut service: RegistrarService,
        owner: AuthPrincipal,
        quota_owner: AuthPrincipal,
        default_resolver: Principal,
        register_years: u32,
        mut mock_registry_api: MockRegistryApi,
        mock_now: u64,
    ) {
        let name = create_test_name("nice");

        let _ctx = mock_registry_api.expect_set_subdomain_owner().returning(
            move |label, parent_name, sub_owner, ttl, resolver| {
                assert_eq!(label, "nice");
                assert_eq!(parent_name, NAMING_TOP_LABEL.to_string());
                assert_eq!(sub_owner, owner.0);
                assert_eq!(ttl, DEFAULT_TTL);
                assert_eq!(resolver, default_resolver);
                Err(NamingError::Unknown.into())
            },
        );
        service.registry_api = Arc::new(mock_registry_api);

        // act
        let context = RegisterCoreContext::new(
            name.to_string(),
            owner,
            register_years,
            TimeInNs(mock_now),
            false,
        );
        let result = service
            .register_with_quota_core(context, &quota_owner, TEST_QUOTA)
            .await;

        // assert
        assert_quota_count(&quota_owner, register_years);
        STATE.with(|s| {
            let store = s.registration_store.borrow();
            assert_eq!(store.get_registrations().borrow().len(), 0);
        });
        assert_eq!(
            result,
            Err(NamingError::RemoteError(NamingError::Unknown.into()))
        );
    }

    #[rstest]
    async fn test_register_success(
        mut service: RegistrarService,
        owner: AuthPrincipal,
        quota_owner: AuthPrincipal,
        default_resolver: Principal,
        register_years: u32,
        mut mock_registry_api: MockRegistryApi,
        mock_now: u64,
    ) {
        let name = create_test_name("nice");
        let api_name = name.clone();

        let _ctx = mock_registry_api.expect_set_subdomain_owner().returning(
            move |label, parent_name, sub_owner, ttl, resolver| {
                assert_eq!(label, "nice");
                assert_eq!(parent_name, NAMING_TOP_LABEL.to_string());
                assert_eq!(sub_owner, owner.0);
                assert_eq!(ttl, DEFAULT_TTL);
                assert_eq!(resolver, default_resolver);
                Ok(RegistryDto {
                    owner: owner.0,
                    name: api_name.to_string(),
                    ttl,
                    resolver,
                })
            },
        );
        service.registry_api = Arc::new(mock_registry_api);

        // act
        let context = RegisterCoreContext::new(
            name.clone(),
            owner,
            register_years,
            TimeInNs(mock_now),
            false,
        );
        let result = service
            .register_with_quota_core(context, &quota_owner, TEST_QUOTA)
            .await;

        // assert
        assert_eq!(result, Ok(true));
        assert_quota_count(&quota_owner, 0);
        STATE.with(|s| {
            let store = s.registration_store.borrow();
            let registrations = store.get_registrations();
            assert_eq!(
                registrations.borrow().get(&name),
                Some(&Registration::new(
                    owner.0,
                    name.clone(),
                    get_expired_at(register_years, TimeInNs(mock_now)).0,
                    mock_now,
                ))
            );
        });
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

mod reclaim_name {
    use super::*;

    #[rstest]
    async fn reclaim_name_success(
        service: RegistrarService,
        mock_now: u64,
        mock_user1: Principal,
        _mock_user2: Principal,
    ) {
        STATE.with(|s| {
            let mut store = s.registration_store.borrow_mut();
            store.add_registration(Registration::new(
                mock_user1.clone(),
                create_test_name("test-name").to_string(),
                mock_now + 1111,
                mock_now,
            ));
        });

        // act
        let reclaim_result = service
            .reclaim_name(&create_test_name("test-name"), &mock_user1)
            .await;

        assert_eq!(reclaim_result.is_ok(), true);
    }

    #[rstest]
    async fn reclaim_name_failed_name_not_found(service: RegistrarService, mock_user1: Principal) {
        // act
        let reclaim_result = service
            .reclaim_name(&create_test_name("test-name"), &mock_user1)
            .await;

        assert_eq!(
            reclaim_result.err().unwrap(),
            NamingError::RegistrationNotFound
        );
    }

    #[rstest]
    async fn reclaim_name_failed_caller_error(
        service: RegistrarService,
        mock_now: u64,
        mock_user1: Principal,
        mock_user2: Principal,
    ) {
        STATE.with(|s| {
            let mut store = s.registration_store.borrow_mut();
            store.add_registration(Registration::new(
                mock_user1.clone(),
                create_test_name("test-name").to_string(),
                mock_now + 1111,
                mock_now,
            ));
        });

        // act
        let reclaim_result = service
            .reclaim_name(&create_test_name("test-name"), &mock_user2)
            .await;

        // assert
        assert_eq!(reclaim_result.err().unwrap(), NamingError::PermissionDenied);
    }
}

mod transfer {
    use common::errors::ErrorInfo;

    use super::*;

    #[rstest]
    async fn test_transfer_success(
        mut service: RegistrarService,
        mut mock_registry_api: MockRegistryApi,
        mock_user1: Principal,
        mock_user2: Principal,
        mock_user3: Principal,
        mock_now: u64,
    ) {
        let test_name = FirstLevelName::from(create_test_name("icnaming"));
        STATE.with(|s| {
            let mut store = s.registration_store.borrow_mut();
            store.add_registration(Registration::new(
                mock_user1.clone(),
                test_name.to_string(),
                mock_now + 1,
                mock_now,
            ));

            let mut store = s.registration_approval_store.borrow_mut();
            store.set_approval(&test_name, &mock_user3, mock_now);
        });

        let api_received_name = test_name.clone();
        let api_received_owner = mock_user2.clone();
        mock_registry_api
            .expect_transfer()
            .returning(move |name, new_owner, _resolver| {
                assert_eq!(name, api_received_name.0.get_name().clone());
                assert_eq!(new_owner, api_received_owner);
                Ok(true)
            });
        service.registry_api = Arc::new(mock_registry_api);

        // act
        let result = service
            .transfer(test_name.0.get_name().as_str(), &mock_user1, mock_user2)
            .await;

        // assert
        assert!(result.is_ok());
        STATE.with(|s| {
            let store = s.registration_store.borrow();
            let registration = store.get_registration(&test_name).unwrap();
            assert_eq!(registration.get_name(), test_name.0.get_name().clone());
            assert_eq!(registration.get_owner(), mock_user2);
            assert_eq!(registration.get_created_at(), mock_now);
            assert_eq!(registration.get_expired_at(), mock_now + 1);

            let store = s.registration_approval_store.borrow();
            assert_eq!(store.has_approved_to(&test_name), false);
        });
    }

    #[rstest]
    async fn test_transfer_failed_name_not_found(
        service: RegistrarService,
        mock_user1: Principal,
        mock_user2: Principal,
    ) {
        // act
        let result = service
            .transfer(&create_test_name("test-name"), &mock_user1, mock_user2)
            .await;

        // assert
        assert_eq!(result.err().unwrap(), NamingError::RegistrationNotFound);
    }

    #[rstest]
    async fn test_transfer_failed_caller_error(
        service: RegistrarService,
        mock_user1: Principal,
        mock_user2: Principal,
        mock_user3: Principal,
    ) {
        STATE.with(|s| {
            let mut store = s.registration_store.borrow_mut();
            store.add_registration(Registration::new(
                mock_user1.clone(),
                create_test_name("test-name").to_string(),
                1,
                0,
            ));
        });

        // act
        let result = service
            .transfer(&create_test_name("test-name"), &mock_user2, mock_user3)
            .await;

        // assert
        assert_eq!(result.err().unwrap(), NamingError::PermissionDenied);
    }

    #[rstest]
    async fn test_transfer_failed_api_error(
        mut service: RegistrarService,
        mut mock_registry_api: MockRegistryApi,
        mock_user1: Principal,
        mock_user2: Principal,
        mock_user3: Principal,
        mock_now: u64,
    ) {
        let test_name = FirstLevelName::from(create_test_name("icnaming"));
        STATE.with(|s| {
            let mut store = s.registration_store.borrow_mut();
            store.add_registration(Registration::new(
                mock_user1.clone(),
                test_name.to_string(),
                mock_now + 1,
                mock_now,
            ));

            let mut store = s.registration_approval_store.borrow_mut();
            store.set_approval(&test_name, &mock_user3, mock_now);
        });

        mock_registry_api
            .expect_transfer()
            .returning(|_name, _new_owner, _resolver| Err(ErrorInfo::from(NamingError::Unknown)));
        service.registry_api = Arc::new(mock_registry_api);

        // act
        let result = service
            .transfer(&create_test_name("icnaming"), &mock_user1, mock_user2)
            .await;

        // assert
        assert_eq!(
            result.err().unwrap(),
            NamingError::RemoteError(ErrorInfo::from(NamingError::Unknown))
        );
    }
}

mod transfer_by_admin {
    use common::permissions::get_admin;

    use super::*;

    #[rstest]
    async fn test_transfer_by_admin_success(
        mut service: RegistrarService,
        mut mock_registry_api: MockRegistryApi,
        mock_user1: Principal,
        mock_user2: Principal,
        mock_user3: Principal,
        mock_now: u64,
    ) {
        let admin = get_admin();
        let test_name = FirstLevelName::from(create_test_name("icnaming"));
        STATE.with(|s| {
            let mut store = s.registration_store.borrow_mut();
            store.add_registration(Registration::new(
                mock_user1.clone(),
                test_name.to_string(),
                mock_now + 1,
                mock_now,
            ));

            let mut store = s.registration_approval_store.borrow_mut();
            store.set_approval(&test_name, &mock_user3, mock_now);
        });

        let api_received_name = test_name.clone();
        let api_received_owner = mock_user2.clone();
        mock_registry_api
            .expect_transfer()
            .returning(move |name, new_owner, _resolver| {
                assert_eq!(name, api_received_name.0.get_name().to_string());
                assert_eq!(new_owner, api_received_owner);
                Ok(true)
            });
        service.registry_api = Arc::new(mock_registry_api);

        // act
        let result = service
            .transfer_by_admin(test_name.0.get_name().as_str(), &admin, mock_user2)
            .await;

        // assert
        assert!(result.is_ok());
        STATE.with(|s| {
            let store = s.registration_store.borrow();
            let registration = store.get_registration(&test_name).unwrap();
            assert_eq!(registration.get_name(), test_name.0.get_name().clone());
            assert_eq!(registration.get_owner(), mock_user2);
            assert_eq!(registration.get_created_at(), mock_now);
            assert_eq!(registration.get_expired_at(), mock_now + 1);

            let store = s.registration_approval_store.borrow();
            assert_eq!(store.has_approved_to(&test_name), false);
        });
    }

    #[rstest]
    #[should_panic]
    async fn test_transfer_by_admin_failed_not_reserved_name(
        service: RegistrarService,
        mock_user1: Principal,
        mock_user2: Principal,
        mock_user3: Principal,
        mock_now: u64,
    ) {
        let admin = get_admin();
        let test_name = FirstLevelName::from(create_test_name("something-not-reserved"));
        STATE.with(|s| {
            let mut store = s.registration_store.borrow_mut();
            store.add_registration(Registration::new(
                mock_user1.clone(),
                test_name.to_string(),
                mock_now + 1,
                mock_now,
            ));

            let mut store = s.registration_approval_store.borrow_mut();
            store.set_approval(&test_name, &mock_user3, mock_now);
        });

        // act
        let _result = service
            .transfer_by_admin(test_name.0.get_name().as_str(), &admin, mock_user2)
            .await;
    }

    #[rstest]
    async fn test_transfer_by_admin_failed_not_admin(
        service: RegistrarService,
        mock_user1: Principal,
        mock_user2: Principal,
    ) {
        let _admin = get_admin();
        let test_name = FirstLevelName::from(create_test_name("icnaming"));

        // act
        let result = service
            .transfer_by_admin(test_name.0.get_name().as_str(), &mock_user1, mock_user2)
            .await;

        // assert
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), NamingError::Unauthorized);
    }
}

mod approve {
    use super::*;

    #[rstest]
    fn test_approve_success(
        service: RegistrarService,
        mock_user1: Principal,
        mock_user2: Principal,
        mock_now: u64,
    ) {
        let test_name_str = create_test_name("icnaming");
        let test_name = FirstLevelName::from(test_name_str.as_str());
        STATE.with(|s| {
            let mut store = s.registration_store.borrow_mut();
            store.add_registration(Registration::new(
                mock_user1.clone(),
                test_name.to_string(),
                mock_now + 1,
                mock_now,
            ));
        });

        // act
        let result = service.approve(&mock_user1, mock_now, &test_name_str, mock_user2.clone());

        // assert
        assert!(result.is_ok());
        STATE.with(|s| {
            let store = s.registration_approval_store.borrow();
            assert_eq!(store.is_approved_to(&test_name, &mock_user2), true);
        });
    }

    #[rstest]
    fn test_approve_success_twice(
        service: RegistrarService,
        mock_user1: Principal,
        mock_user2: Principal,
        mock_user3: Principal,
        mock_now: u64,
    ) {
        let test_name_str = create_test_name("icnaming");
        let test_name = FirstLevelName::from(test_name_str.as_str());
        STATE.with(|s| {
            let mut store = s.registration_store.borrow_mut();
            store.add_registration(Registration::new(
                mock_user1.clone(),
                test_name.to_string(),
                mock_now + 1,
                mock_now,
            ));
        });

        // act
        let _result = service.approve(&mock_user1, mock_now, &test_name_str, mock_user2.clone());
        let result = service.approve(&mock_user1, mock_now, &test_name_str, mock_user3.clone());

        // assert
        assert!(result.is_ok());
        STATE.with(|s| {
            let store = s.registration_approval_store.borrow();
            assert_eq!(store.is_approved_to(&test_name, &mock_user3), true);
        });
    }

    #[rstest]
    fn test_approve_success_remove_approval(
        service: RegistrarService,
        mock_user1: Principal,
        mock_user2: Principal,
        mock_now: u64,
    ) {
        let test_name_str = create_test_name("icnaming");
        let test_name = FirstLevelName::from(test_name_str.as_str());
        STATE.with(|s| {
            let mut store = s.registration_store.borrow_mut();
            store.add_registration(Registration::new(
                mock_user1.clone(),
                test_name_str.to_string(),
                mock_now + 1,
                mock_now,
            ));
        });

        // act
        let _result = service.approve(&mock_user1, mock_now, &test_name_str, mock_user2.clone());
        let result = service.approve(
            &mock_user1,
            mock_now,
            &test_name_str,
            Principal::anonymous(),
        );

        // assert
        assert!(result.is_ok());
        STATE.with(|s| {
            let store = s.registration_approval_store.borrow();
            assert_eq!(store.has_approved_to(&test_name), false);
        });
    }

    #[rstest]
    fn test_approve_failed_is_not_owner(
        service: RegistrarService,
        mock_user1: Principal,
        mock_user2: Principal,
        mock_now: u64,
    ) {
        let test_name_str = create_test_name("icnaming");
        let test_name = FirstLevelName::from(test_name_str.as_str());
        STATE.with(|s| {
            let mut store = s.registration_store.borrow_mut();
            store.add_registration(Registration::new(
                mock_user1.clone(),
                test_name.to_string(),
                mock_now + 1,
                mock_now,
            ));
        });

        // act
        let result = service.approve(&mock_user2, mock_now, &test_name_str, mock_user1.clone());

        // assert
        assert!(result.is_err());
        STATE.with(|s| {
            let store = s.registration_approval_store.borrow();
            assert_eq!(store.has_approved_to(&test_name), false);
        });
    }
}

mod transfer_from {
    use super::*;

    #[rstest]
    async fn test_transfer_from_success(
        mut service: RegistrarService,
        mut mock_registry_api: MockRegistryApi,
        mock_user1: Principal,
        mock_user2: Principal,
        mock_now: u64,
    ) {
        let test_name_str = create_test_name("icnaming");
        let test_name = FirstLevelName::from(test_name_str.as_str());
        STATE.with(|s| {
            let mut store = s.registration_store.borrow_mut();
            store.add_registration(Registration::new(
                mock_user1.clone(),
                test_name.to_string(),
                mock_now + 1,
                mock_now,
            ));
        });
        service
            .approve(&mock_user1, mock_now, &test_name_str, mock_user2.clone())
            .unwrap();

        let api_received_name = test_name_str.clone();
        let api_received_owner = mock_user2.clone();
        mock_registry_api
            .expect_transfer()
            .returning(move |name, new_owner, _resolver| {
                assert_eq!(name, api_received_name);
                assert_eq!(new_owner, api_received_owner);
                Ok(true)
            });
        service.registry_api = Arc::new(mock_registry_api);

        // act
        let result = service.transfer_from(&mock_user2, &test_name_str).await;

        // assert
        assert!(result.is_ok());
        STATE.with(|s| {
            let store = s.registration_store.borrow();
            let registration = store.get_registration(&test_name).unwrap();
            assert_eq!(registration.get_owner(), mock_user2);
        });
    }

    #[rstest]
    async fn test_transfer_from_failed_not_approve(
        service: RegistrarService,
        _mock_registry_api: MockRegistryApi,
        mock_user1: Principal,
        mock_user2: Principal,
        mock_now: u64,
    ) {
        let test_name_str = create_test_name("icnaming");
        let test_name = FirstLevelName::from(test_name_str.as_str());
        STATE.with(|s| {
            let mut store = s.registration_store.borrow_mut();
            store.add_registration(Registration::new(
                mock_user1.clone(),
                test_name.to_string(),
                mock_now + 1,
                mock_now,
            ));
        });

        // act
        let result = service
            .transfer_from(&mock_user2, test_name_str.as_str())
            .await;

        // assert
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), NamingError::PermissionDenied);
    }
}

mod transfer_from_quota {
    use super::*;

    #[rstest]
    fn test_transfer_from_quota_success(
        service: RegistrarService,
        mock_user1: Principal,
        quota_owner: AuthPrincipal,
        register_years: u32,
    ) {
        let marketplace = get_named_get_canister_id(CanisterNames::NamingMarketplace);
        let result = service.transfer_from_quota(
            &marketplace,
            quota_owner.0,
            mock_user1,
            TEST_QUOTA,
            register_years,
        );

        // assert
        assert!(result.is_ok());
        assert_quota_count(&quota_owner, 0);
        assert_quota_count(&AuthPrincipal(mock_user1), register_years);
    }

    #[rstest]
    fn test_transfer_from_quota_success_2(
        service: RegistrarService,
        mock_user1: Principal,
        quota_owner: AuthPrincipal,
        register_years: u32,
    ) {
        let marketplace = get_named_get_canister_id(CanisterNames::NamingMarketplace);
        let result = service.transfer_from_quota(
            &marketplace,
            quota_owner.0,
            mock_user1,
            TEST_QUOTA,
            register_years - 1,
        );

        // assert
        assert!(result.is_ok());
        assert_quota_count(&quota_owner, 1);
        assert_quota_count(&AuthPrincipal(mock_user1), register_years - 1);
    }

    #[rstest]
    fn test_transfer_from_quota_failed(
        service: RegistrarService,
        mock_user1: Principal,
        quota_owner: AuthPrincipal,
        register_years: u32,
    ) {
        let marketplace = get_named_get_canister_id(CanisterNames::NamingMarketplace);
        let result = service.transfer_from_quota(
            &marketplace,
            mock_user1,
            quota_owner.0,
            TEST_QUOTA,
            register_years - 1,
        );

        // assert
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert_eq!(error, NamingError::InsufficientQuota);
        assert_quota_count(&quota_owner, register_years);
        assert_quota_count(&AuthPrincipal(mock_user1), 0);
    }
}

mod get_expired_at {
    use super::*;

    #[rstest]
    #[case(1651384898_000_000000u64, 3, 1746057600000000000u64)]
    #[case(1651384898_000_000000u64, 10, 1966982400000000000u64)]
    fn test_nice(#[case] now: u64, #[case] years: u32, #[case] expected: u64) {
        let result = get_expired_at(years, TimeInNs(now));
        assert_eq!(result, TimeInNs(expected));
    }
}

mod set_record_value {
    use super::*;

    #[rstest]
    async fn set_record_value_registration_count_gt_1(
        mut service: RegistrarService,
        mock_user1: Principal,
        mut mock_resolver_api: MockResolverApi,
    ) {
        mock_resolver_api.expect_set_record_value().returning(
            |_name: String, _patch_values: HashMap<String, String>| {
                // assert _patch_values.len() == 2
                assert_eq!(2, _patch_values.len());
                Ok(true)
            },
        );
        service.resolver_api = Arc::new(mock_resolver_api);
        let name = "test.ic";
        let result = service
            .set_record_value(name.to_string(), &mock_user1, 2)
            .await;
        assert!(result.is_ok());
    }

    #[rstest]
    async fn set_record_value_registration_count_eq_1(
        mut service: RegistrarService,
        mock_user1: Principal,
        mut mock_resolver_api: MockResolverApi,
    ) {
        mock_resolver_api.expect_set_record_value().returning(
            |_name: String, _patch_values: HashMap<String, String>| {
                // assert _patch_values.len() == 3
                assert_eq!(3, _patch_values.len());
                Ok(true)
            },
        );
        service.resolver_api = Arc::new(mock_resolver_api);
        let name = "test.ic";
        let result = service
            .set_record_value(name.to_string(), &mock_user1, 1)
            .await;
        assert!(result.is_ok());
    }
}

mod nft_query_service {
    use super::*;
    use candid::decode_args;
    use common::token_identifier::{encode_token_id, TokenIndex};
    use std::string::String;

    #[rstest]
    fn invalid_canister_id(mock_user1: Principal, mock_now: u64) {
        let call_context = CallContext {
            caller: mock_user1,
            now: TimeInNs(mock_now),
        };
        let result = call_context.must_be_canister_id();
        assert!(result.is_err());
    }

    #[rstest]
    fn get_registry(mut service: RegistrarService) {
        let test_name_str = create_test_name("icnaming");
        STATE.with(|s| {
            let mut store = s.token_index_store.borrow_mut();
            store.try_add_registration_name(RegistrationName(test_name_str.to_string()));
        });
        let result = service.get_registry();
        assert_eq!(result.len(), 1);
    }

    #[rstest]
    fn get_tokens(mut service: RegistrarService) {
        let test_name_str1 = create_test_name("icnaming1");
        let test_name_str2 = create_test_name("icnaming2");
        STATE.with(|s| {
            let mut store = s.token_index_store.borrow_mut();
            store.try_add_registration_name(RegistrationName(test_name_str1.to_string()));
            store.try_add_registration_name(RegistrationName(test_name_str2.to_string()));
        });

        let mut result = service.get_tokens();
        assert_eq!(result.len(), 2);
        result.sort_by(|a, b| a.0.cmp(&b.0));
        let first_token = result.first().unwrap().to_owned();
        let last_token = result.last().unwrap().to_owned();
        match first_token.1 {
            Metadata::NonFungible(registration) => {
                let (metadata,): (HashMap<String, String>,) =
                    decode_args(&registration.metadata.unwrap()).unwrap();
                assert_eq!(metadata.get("name").unwrap(), &test_name_str1);
            }
            _ => {
                panic!("Expected registration");
            }
        }
        match last_token.1 {
            Metadata::NonFungible(registration) => {
                let (metadata,): (HashMap<String, String>,) =
                    decode_args(&registration.metadata.unwrap()).unwrap();
                assert_eq!(metadata.get("name").unwrap(), &test_name_str2);
            }
            _ => {
                panic!("Expected registration");
            }
        }
    }

    #[rstest]
    fn metadata(mut service: RegistrarService, mock_canister1: Principal, mock_now: u64) {
        let test_name_str = create_test_name("icnaming");
        STATE.with(|s| {
            let mut store = s.token_index_store.borrow_mut();
            store.try_add_registration_name(RegistrationName(test_name_str.to_string()));
            let mut store = s.registration_store.borrow_mut();
            store.add_registration(Registration::new(
                mock_canister1.clone(),
                test_name_str.to_string(),
                mock_now + 1,
                mock_now,
            ));
        });
        let call_context = CallContext {
            caller: mock_canister1,
            now: TimeInNs(mock_now),
        };
        let canister_id = call_context.must_be_canister_id().unwrap();
        let token_id = encode_token_id(canister_id, TokenIndex(1u32));
        let result = service.metadata(
            &CallContext {
                caller: mock_canister1,
                now: TimeInNs(mock_now),
            },
            token_id,
        );
        println!("{:?}", result);
        assert!(result.is_ok());
        let result = result.unwrap();
        match result {
            Metadata::NonFungible(registration) => {
                let (metadata,): (HashMap<String, String>,) =
                    decode_args(&registration.metadata.unwrap()).unwrap();
                assert_eq!(metadata.get("name").unwrap(), &test_name_str);
            }
            _ => {
                panic!("Expected registration");
            }
        }
    }

    #[rstest]
    fn get_supply(mut service: RegistrarService) {
        let test_name_str = create_test_name("icnaming");
        STATE.with(|s| {
            let mut store = s.token_index_store.borrow_mut();
            store.try_add_registration_name(RegistrationName(test_name_str.to_string()));
        });
        let result = service.get_supply();
        assert!(result.is_ok());
        let result = result.unwrap();

        assert_eq!(result, 1u128);
    }

    #[rstest]
    fn get_supply_default(mut service: RegistrarService) {
        let result = service.get_supply();
        assert!(result.is_ok());
        let result = result.unwrap();

        assert_eq!(result, 0u128);
    }
}

// mod load_state {
//     use std::fs::File;
//     use std::io::Write;
//     use common::dto::decode_zlib;
//     use common::state::StableState;
//     use super::*;
//
//     #[rstest]
//     fn get_registration_owners(
//         service: RegistrarService,
//     ) {
//         let zlib = include_bytes!("../../../../local_state_data/registrar/latest.zlib");
//         let candi = decode_zlib(zlib);
//         let state = State::decode(candi).unwrap();
//         let store = state.registration_store.borrow();
//         // out to file registrar.csv
//         let mut wtr = csv::Writer::from_writer(vec![]);
//         for (name, registration) in store.get_registrations() {
//             wtr.serialize((name, registration.get_owner().to_string())).unwrap();
//         }
//         wtr.flush().unwrap();
//         let csv = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
//         File::create("registrar.csv").unwrap().write_all(csv.as_bytes()).unwrap();
//     }
// }

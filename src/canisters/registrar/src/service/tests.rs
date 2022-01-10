use std::sync::Arc;

use candid::Principal;
use rstest::*;

use common::state::{add_principal, set_named_principal_owner};
use test_common::canister_api::*;
use test_common::ic_api::init_test;

use super::*;

const TEST_QUOTA: QuotaType = QuotaType::LenGte(4);

#[fixture]
fn owner() -> Principal {
    Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap()
}

#[fixture]
fn quota_owner() -> Principal {
    Principal::from_text("5i47k-cqaaa-aaaak-qaddq-cai").unwrap()
}

#[fixture]
fn default_resolver() -> Principal {
    owner()
}

#[fixture]
fn service(_init_test: (), quota_owner: Principal) -> RegistrarService {
    USER_QUOTA_MANAGER.with(|m| {
        let mut m = m.borrow_mut();
        m.add_quota(quota_owner, TEST_QUOTA, 1);
    });
    let service = RegistrarService::new();
    service
}

fn assert_quota_count(quota_owner: &Principal, count: u32) {
    USER_QUOTA_MANAGER.with(|m| {
        let m = m.borrow();
        assert_eq!(m.get_quota(quota_owner, &TEST_QUOTA).unwrap_or(0), count);
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
    Err(format!("top level of name must be {}", TOP_LABEL))
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
    fn test_validate_name(
        service: RegistrarService,
        #[case] input: &str,
        #[case] expected: Result<NameParseResult, String>,
    ) {
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
        USER_QUOTA_MANAGER.with(|m| {
            let mut m = m.borrow_mut();
            m.add_quota(owner.clone(), quota_type.clone(), 1);
        });
        let result = service.validate_quota(&name, &owner, &quota_type);
        assert_eq!(result, expected);
    }

    #[rstest]
    fn test_validate_quota_no_quota(service: RegistrarService, owner: Principal) {
        let name = NameParseResult::parse("nice.icp");
        let quota_type = QuotaType::LenGte(3);
        let result = service.validate_quota(&name, &owner, &quota_type);
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
            REGISTRATIONS.with(|registrations| {
                registrations.borrow_mut().insert(
                    name.to_string(),
                    Registration::new(Principal::anonymous(), name.to_string(), 0, 0),
                );
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
            let expired_at = 123;
            REGISTRATIONS.with(|registrations| {
                registrations.borrow_mut().insert(
                    name.to_string(),
                    Registration::new(Principal::anonymous(), name.to_string(), expired_at, 0),
                );
            });
            let expires = service.get_name_expires(name);
            assert_eq!(expires, Ok(expired_at));
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
            Err(ICNSError::InvalidOwner) => {
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
        quota_owner: Principal,
    ) {
        let name = "www.nice.icp";
        let _year = 0;
        let result = service
            .register(name, &owner, 0, 0, &quota_owner, TEST_QUOTA)
            .await;
        assert_quota_count(&quota_owner, 1);
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
    ) {
        let owner = Principal::anonymous();
        let name = "nice.icp";
        let year = 0;
        let result = service
            .register(name, &owner, year, 0, &quota_owner, TEST_QUOTA)
            .await;
        assert_quota_count(&quota_owner, 1);
        assert_eq!(result, Err(ICNSError::InvalidOwner));
    }

    #[rstest]
    async fn test_register_err_lack_of_quota(
        mut service: RegistrarService,
        owner: Principal,
        quota_owner: Principal,
    ) {
        let name = "nice.icp";
        let year = 1;
        USER_QUOTA_MANAGER.with(|quota_manager| {
            let mut quota_manager = quota_manager.borrow_mut();
            quota_manager.sub_quota(&quota_owner.to_owned(), &TEST_QUOTA, 1);
        });

        // act
        let result = service
            .register(name, &owner, year, 0, &quota_owner, TEST_QUOTA)
            .await;
        assert_quota_count(&quota_owner, 0);
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
    ) {
        let name = "nice.icp";
        let year = 0;
        let result = service
            .register(name, &owner, year, 0, &quota_owner, TEST_QUOTA)
            .await;
        assert_quota_count(&quota_owner, 1);
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
    ) {
        let name = "nice.icp";
        let year = 0;
        REGISTRATIONS.with(|registrations| {
            registrations.borrow_mut().insert(
                name.to_string(),
                Registration::new(owner, name.to_string(), 0, 0),
            );
        });
        let result = service
            .register(name, &owner, year, 0, &quota_owner, TEST_QUOTA)
            .await;
        assert_quota_count(&quota_owner, 1);
        assert_eq!(result, Err(ICNSError::RegistrationHasBeenTaken));
    }

    #[rstest]
    async fn test_register_err_reserved(
        mut service: RegistrarService,
        owner: Principal,
        quota_owner: Principal,
    ) {
        let name = "icnaming.icp";
        let year = 0;
        let result = service
            .register(name, &owner, year, 0, &quota_owner, TEST_QUOTA)
            .await;
        assert_quota_count(&quota_owner, 1);
        assert_eq!(result, Err(ICNSError::RegistrationHasBeenTaken));
    }

    #[fixture]
    fn setup_resolver_canister_name() {
        let owner = owner();
        let default_resolver = default_resolver();
        set_named_principal_owner(&owner, &owner).unwrap();
        add_principal(CANISTER_NAME_RESOLVER, &owner, &default_resolver).unwrap();
    }

    #[rstest]
    async fn test_register_api_failed(
        _setup_resolver_canister_name: (),
        mut service: RegistrarService,
        owner: Principal,
        quota_owner: Principal,
        default_resolver: Principal,
        mut mock_registry_api: MockRegistryApi,
    ) {
        let name = "nice.icp";
        let year = 5;
        let now = 0;

        let _ctx = mock_registry_api.expect_set_subdomain_owner().returning(
            move |label, parent_name, sub_owner, ttl, resolver| {
                assert_eq!(label, "nice");
                assert_eq!(parent_name, "icp");
                assert_eq!(sub_owner, owner);
                assert_eq!(ttl, DEFAULT_TTL);
                assert_eq!(resolver, default_resolver);
                Err(ICNSError::Unknown.into())
            },
        );
        service.registry_api = Arc::new(mock_registry_api);

        // act
        let result = service
            .register(name, &owner, year, now, &quota_owner, TEST_QUOTA)
            .await;

        // assert
        assert_quota_count(&quota_owner, 1);
        REGISTRATIONS.with(|registrations| {
            assert_eq!(registrations.borrow().len(), 0);
        });
        assert_eq!(
            result,
            Err(ICNSError::RemoteError(ICNSError::Unknown.into()))
        );
    }

    #[rstest]
    async fn test_register_success(
        _setup_resolver_canister_name: (),
        mut service: RegistrarService,
        owner: Principal,
        quota_owner: Principal,
        default_resolver: Principal,
        mut mock_registry_api: MockRegistryApi,
    ) {
        let name = "nice.icp";
        let year = 5;
        let now = 0;

        let _ctx = mock_registry_api.expect_set_subdomain_owner().returning(
            move |label, parent_name, sub_owner, ttl, resolver| {
                assert_eq!(label, "nice");
                assert_eq!(parent_name, "icp");
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
            .register(name, &owner, year, now, &quota_owner, TEST_QUOTA)
            .await;

        // assert
        assert_quota_count(&quota_owner, 0);
        REGISTRATIONS.with(|registrations| {
            assert_eq!(
                registrations.borrow().get(&name.to_string()),
                Some(&Registration::new(
                    owner,
                    name.to_string(),
                    now + year_to_ms(year),
                    now,
                ))
            );
        });
        assert_eq!(result, Ok(true));
    }
}

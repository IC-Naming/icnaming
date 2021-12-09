use std::sync::Arc;

use candid::Principal;
use rstest::*;

use common::state::{add_principal, set_named_principal_owner};
use test_common::canister_api::*;
use test_common::ic_api::init_test;

use super::*;

#[fixture]
fn service() -> RegistrarService {
    let service = RegistrarService::new();
    service
}

#[fixture]
fn owner() -> Principal {
    Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap()
}

#[fixture]
fn default_resolver() -> Principal {
    owner()
}

mod normalized {
    use super::*;

    #[rstest]
    #[case("test", "test")]
    #[case("123", "123")]
    #[case(" trim_blank ", "trim_blank")]
    #[case(" TOLOWER ", "tolower")]
    #[case(" 你好 ", "你好")]
    fn test_normalized(
        _init_test: (),
        service: RegistrarService,
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        let normalized = service.normalize_name(input);
        assert_eq!(normalized, expected);
    }
}

mod validate_name {
    use super::*;

    #[rstest]
    #[trace]
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
    #[case("nic.icp",
    Err("second level name must be more than 3 characters".to_string())
    )]
    #[case("nic%.icp",
    Err("name must be alphanumeric or -".to_string()),
    )]
    fn test_validate_name(
        _init_test: (),
        service: RegistrarService,
        #[case] input: &str,
        #[case] expected: Result<NameParseResult, String>,
    ) {
        let result = service.validate_name(input);
        match result {
            Ok(parse_result) => {
                assert_eq!(parse_result, expected.unwrap());
            }
            Err(message) => {
                assert_eq!(message, expected.unwrap_err());
            }
        }
    }
}

mod available {
    use super::*;

    #[rstest]
    fn test_available(_init_test: (), service: RegistrarService) {
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
    }
}

mod get_name_expires {
    use super::*;

    #[rstest]
    fn test_get_name_expires(_init_test: (), service: RegistrarService) {
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
    fn test_get_names_invalid_owner(_init_test: (), service: RegistrarService) {
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
    fn test_get_names_invalid_page(_init_test: (), service: RegistrarService) {
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
        _init_test: (),
        mut service: RegistrarService,
        owner: Principal,
    ) {
        let name = "www.nice.icp";
        let _year = 0;
        let result = service.register(name, &owner, 0, 0).await;
        assert_eq!(
            result,
            Err(ICNSError::InvalidName {
                reason: "it must be second level name".to_string()
            })
        );
    }

    #[rstest]
    async fn test_register_err_owner_invalid(_init_test: (), mut service: RegistrarService) {
        let owner = Principal::anonymous();
        let name = "nice.icp";
        let _year = 0;
        let result = service.register(name, &owner, 0, 0).await;
        assert_eq!(result, Err(ICNSError::InvalidOwner));
    }

    #[rstest]
    async fn test_register_err_year_invalid(
        _init_test: (),
        mut service: RegistrarService,
        owner: Principal,
    ) {
        let name = "nice.icp";
        let year = 0;
        let result = service.register(name, &owner, year, 0).await;
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
        _init_test: (),
        mut service: RegistrarService,
        owner: Principal,
    ) {
        let name = "nice.icp";
        let year = 0;
        REGISTRATIONS.with(|registrations| {
            registrations.borrow_mut().insert(
                name.to_string(),
                Registration::new(owner, name.to_string(), 0, 0),
            );
        });
        let result = service.register(name, &owner, year, 0).await;
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
        _init_test: (),
        _setup_resolver_canister_name: (),
        mut service: RegistrarService,
        owner: Principal,
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
        let result = service.register(name, &owner, year, now).await;

        // assert
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
        _init_test: (),
        _setup_resolver_canister_name: (),
        mut service: RegistrarService,
        owner: Principal,
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
        let result = service.register(name, &owner, year, now).await;

        // assert
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

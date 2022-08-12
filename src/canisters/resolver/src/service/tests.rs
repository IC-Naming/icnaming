use std::collections::HashSet;

use candid::Principal;
use common::TimeInNs;
use rstest::*;

use common::constants::*;
use common::dto::RegistryUsers;
use test_common::canister_api::mock_registry_api;
use test_common::canister_api::MockRegistryApi;
use test_common::ic_api::init_test;
use test_common::user::*;

use super::*;

fn add_test_resolver(name: &str) {
    STATE.with(|s| {
        let mut store = s.resolver_store.borrow_mut();
        let resolvers = store.get_resolvers_mut();
        let mut resolver = Resolver::new(name.to_string());
        resolver.set_record_value(RESOLVER_KEY_GITHUB.to_string(), "icns".to_string());
        resolver.set_record_value(RESOLVER_KEY_TWITTER.to_string(), "twitter".to_string());
        resolvers.insert(name.to_string(), resolver);
    });
}

#[fixture]
fn service() -> ResolverService {
    let service = ResolverService::default();
    service
}

mod ensure_resolver_created {
    use super::*;

    #[rstest]
    fn test_ensure_resolver_created(_init_test: (), mut service: ResolverService) {
        let name = "nice.ic";

        // act
        let result = service.ensure_resolver_created(name);
        assert!(result.is_ok());

        // assert
        STATE.with(|s| {
            let store = s.resolver_store.borrow();
            let resolvers = store.get_resolvers();
            assert_eq!(resolvers.len(), 1);
            assert_eq!(resolvers.get(name).unwrap().get_name(), name);
        });
    }

    #[rstest]
    fn test_ensure_resolver_created_already_exists(_init_test: (), mut service: ResolverService) {
        let name = "nice.ic";

        // act
        service.ensure_resolver_created(name).unwrap();
        let result = service.ensure_resolver_created(name);
        assert!(result.is_ok());
        let result = service.ensure_resolver_created(name);

        // assert
        assert!(result.is_ok());
        STATE.with(|s| {
            let store = s.resolver_store.borrow();
            let resolvers = store.get_resolvers();
            assert_eq!(resolvers.len(), 1);
            assert_eq!(resolvers.get(name).unwrap().get_name(), name);
        });
    }
}

mod get_record_value {
    use super::*;

    #[rstest]
    fn test_get_record_value_not_found(_init_test: (), service: ResolverService) {
        let name = "nice.ic";

        // act
        let result = service.get_record_value(name);

        // assert
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 0);
    }

    #[rstest]
    fn test_get_record_value_found(_init_test: (), service: ResolverService) {
        let name = "nice.ic";
        add_test_resolver(name);

        // act
        let result = service.get_record_value(name);

        // assert
        let map = result.unwrap();
        assert_eq!(map.len(), 2);
        assert!(map.contains_key(RESOLVER_KEY_GITHUB));
    }
}

mod set_record_validation {
    use common::named_canister_ids::NAMED_CANISTER_IDS;
    use super::*;

    #[rstest]
    async fn test_set_record_validation_key_too_long(
        _init_test: (),
        mock_now: u64,
        mock_user1: Principal,
    ) {
        let name = "nice.ic";
        let mut patch_values: HashMap<String, String> = HashMap::new();
        let invalid_resolver_key = "a".repeat(RESOLVER_KEY_MAX_LENGTH + 1);
        patch_values.insert(invalid_resolver_key.to_string(), "icns".to_string());
        // add resolver
        add_test_resolver(name);

        // act
        let result = SetRecordValueValidator::new(
            must_not_anonymous(&mock_user1).unwrap(),
            name.to_string(),
            patch_values,
            Resolver::new(name.to_string()),
        )
            .validate()
            .await;

        // assert
        assert!(result.is_err());
        match result {
            Err(e) => {
                assert_eq!(
                    e,
                    NamingError::KeyMaxLengthError {
                        max: RESOLVER_KEY_MAX_LENGTH,
                    }
                );
            }
            _ => assert!(false),
        }
    }

    #[rstest]
    async fn test_set_record_validation_value_invalid(
        _init_test: (),
        mock_now: u64,
        mock_user1: Principal,
    ) {
        let name = "nice.ic";
        let mut patch_values: HashMap<String, String> = HashMap::new();
        let mut value = String::new();
        for _ in 0..(RESOLVER_VALUE_MAX_LENGTH + 1) {
            value.push('a');
        }
        patch_values.insert(RESOLVER_KEY_GITHUB.to_string(), value);

        // add resolver
        add_test_resolver(name);

        // act
        let result = SetRecordValueValidator::new(
            must_not_anonymous(&mock_user1).unwrap(),
            name.to_string(),
            patch_values,
            Resolver::new(name.to_string()),
        )
            .validate()
            .await;

        // assert
        assert!(result.is_err());
        match result {
            Err(e) => {
                assert_eq!(
                    e,
                    NamingError::ValueMaxLengthError {
                        max: RESOLVER_VALUE_MAX_LENGTH
                    }
                );
            }
            _ => assert!(false),
        }
    }

    #[rstest]
    async fn test_set_record_validation_too_many_items(
        _init_test: (),
        mock_now: u64,
        mock_user1: Principal,
    ) {
        let name = "nice.ic";
        let mut patch_values: HashMap<String, String> = HashMap::new();
        patch_values.insert(RESOLVER_KEY_GITHUB.to_string(), "icns".to_string());
        // add resolver
        add_test_resolver(name);

        // act
        let mut resolver = Resolver::new(name.to_string());
        for i in 0..RESOLVER_ITEM_MAX_COUNT {
            resolver.set_record_value(format!("{}", i), format!("{}", i));
        }
        let result = SetRecordValueValidator::new(
            must_not_anonymous(&mock_user1).unwrap(),
            name.to_string(),
            patch_values,
            resolver,
        )
            .validate()
            .await;

        // assert
        assert!(result.is_err());
        match result {
            Err(e) => {
                assert_eq!(
                    e,
                    NamingError::TooManyResolverKeys {
                        max: RESOLVER_ITEM_MAX_COUNT as u32,
                    }
                );
            }
            _ => assert!(false),
        }
    }

    #[rstest]
    async fn test_set_record_validation_permission_deny(
        _init_test: (),
        mut mock_registry_api: MockRegistryApi,
        mock_now: u64,
        mock_user1: Principal,
    ) {
        let name = "nice.ic";
        let mut patch_values: HashMap<String, String> = HashMap::new();
        patch_values.insert(RESOLVER_KEY_GITHUB.to_string(), "icns".to_string());
        let owner = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();

        // add resolver
        add_test_resolver(name);

        let _ctx = mock_registry_api
            .expect_get_users()
            .returning(move |_name| {
                Ok(RegistryUsers {
                    owner,
                    operators: HashSet::new(),
                })
            });

        // act
        let mut context = SetRecordValueValidator::new(
            must_not_anonymous(&mock_user1).unwrap(),
            name.to_string(),
            patch_values,
            Resolver::new(name.to_string()),
        );
        context.registry_api = Arc::new(mock_registry_api);

        let result = context.validate().await;

        // assert
        assert!(result.is_err());
        match result {
            Err(e) => {
                assert_eq!(e, NamingError::PermissionDenied {});
            }
            _ => assert!(false),
        }
    }

    #[rstest]
    async fn test_set_record_validation_success(
        _init_test: (),
        mut service: ResolverService,
        mut mock_registry_api: MockRegistryApi,
        mock_now: u64,
        mock_user1: Principal,
    ) {
        let name = "nice.ic";
        let mut patch_values: HashMap<String, String> = HashMap::new();
        let icp_addr = "rrkah-fqaaa-aaaaa-aaaaq-cai";
        patch_values.insert(RESOLVER_KEY_ICP.to_string(), icp_addr.to_string());
        // enter blank value to remove the key
        patch_values.insert(RESOLVER_KEY_TWITTER.to_string(), "".to_string());
        let owner = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();

        // add resolver
        add_test_resolver(name);

        let _ctx = mock_registry_api
            .expect_get_users()
            .returning(move |_name| {
                Ok(RegistryUsers {
                    owner,
                    operators: HashSet::new(),
                })
            });

        // act
        let mut context = SetRecordValueValidator::new(
            must_not_anonymous(&owner).unwrap(),
            name.to_string(),
            patch_values,
            Resolver::new(name.to_string()),
        );
        context.registry_api = Arc::new(mock_registry_api);

        let result = context.validate().await;

        // assert
        assert!(result.is_ok(), "{:?}", result);
        let result = result.unwrap();
        assert_eq!(
            result.update_primary_name_input,
            UpdatePrimaryNameInput::DoNothing
        );
        assert_eq!(result.update_records_input.len(), 2);
        assert_eq!(
            result.update_records_input.get(RESOLVER_KEY_ICP).unwrap(),
            &UpdateRecordInput::Set(icp_addr.to_string())
        );
        assert_eq!(
            result
                .update_records_input
                .get(RESOLVER_KEY_TWITTER)
                .unwrap(),
            &UpdateRecordInput::Remove
        );
    }

    #[rstest]
    async fn test_set_record_validation_update_primary_name(
        _init_test: (),
        mut service: ResolverService,
        mut mock_registry_api: MockRegistryApi,
        mock_now: u64,
        mock_user1: Principal,
    ) {
        let name = "nice.ic";
        let mut patch_values: HashMap<String, String> = HashMap::new();
        let icp_addr = "rrkah-fqaaa-aaaaa-aaaaq-cai";
        patch_values.insert(
            RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL.to_string(),
            icp_addr.to_string(),
        );
        // enter blank value to remove the key
        patch_values.insert(RESOLVER_KEY_TWITTER.to_string(), "".to_string());
        let owner = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();

        // add resolver
        add_test_resolver(name);

        let _ctx = mock_registry_api
            .expect_get_users()
            .returning(move |_name| {
                Ok(RegistryUsers {
                    owner,
                    operators: HashSet::new(),
                })
            });

        // act
        let mut context = SetRecordValueValidator::new(
            must_not_anonymous(&owner).unwrap(),
            name.to_string(),
            patch_values,
            Resolver::new(name.to_string()),
        );
        context.registry_api = Arc::new(mock_registry_api);

        let result = context.validate().await;

        // assert
        assert!(result.is_ok(), "{:?}", result);
        let result = result.unwrap();
        assert_eq!(
            result.update_primary_name_input,
            UpdatePrimaryNameInput::Set(owner.clone())
        );
    }

    #[rstest]
    async fn test_set_record_validation_remove_primary_name(
        _init_test: (),
        mut service: ResolverService,
        mut mock_registry_api: MockRegistryApi,
        mock_now: u64,
        mock_user1: Principal,
    ) {
        let name = "nice.ic";
        let mut patch_values: HashMap<String, String> = HashMap::new();
        patch_values.insert(
            RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL.to_string(),
            "".to_string(),
        );
        // enter blank value to remove the key
        patch_values.insert(RESOLVER_KEY_TWITTER.to_string(), "".to_string());
        let owner = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();

        // add resolver
        add_test_resolver(name);

        let _ctx = mock_registry_api
            .expect_get_users()
            .returning(move |_name| {
                Ok(RegistryUsers {
                    owner,
                    operators: HashSet::new(),
                })
            });

        // act
        let mut context = SetRecordValueValidator::new(
            must_not_anonymous(&owner).unwrap(),
            name.to_string(),
            patch_values,
            Resolver::new(name.to_string()),
        );
        context.registry_api = Arc::new(mock_registry_api);

        let result = context.validate().await;

        // assert
        assert!(result.is_ok(), "{:?}", result);
        let result = result.unwrap();
        assert_eq!(
            result.update_primary_name_input,
            UpdatePrimaryNameInput::Remove(owner.clone())
        );
    }

    #[rstest]
    async fn test_set_record_validation_operator_cannot_update_primary_name(
        _init_test: (),
        mut mock_registry_api: MockRegistryApi,
        mock_user1: Principal,
    ) {
        let name = "nice.ic";
        let mut patch_values: HashMap<String, String> = HashMap::new();
        patch_values.insert(
            RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL.to_string(),
            "".to_string(),
        );
        // enter blank value to remove the key
        patch_values.insert(RESOLVER_KEY_TWITTER.to_string(), "".to_string());
        let owner = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();

        // add resolver
        add_test_resolver(name);

        let _ctx = mock_registry_api
            .expect_get_users()
            .returning(move |_name| {
                let mut set = HashSet::new();
                set.insert(mock_user1.clone());
                Ok(RegistryUsers {
                    owner,
                    operators: set,
                })
            });

        // act
        let mut context = SetRecordValueValidator::new(
            must_not_anonymous(&mock_user1).unwrap(),
            name.to_string(),
            patch_values,
            Resolver::new(name.to_string()),
        );
        context.registry_api = Arc::new(mock_registry_api);

        let result = context.validate().await;

        // assert
        assert!(result.is_err());
        let result = result.err().unwrap();
        assert_eq!(result, NamingError::PermissionDenied);
    }

    #[rstest]
    async fn test_set_record_validation_call_from_canister_registrar_should_pass(_init_test: (),
                                                                        mut mock_registry_api: MockRegistryApi) {
        let name = "nice.ic";
        let mut patch_values: HashMap<String, String> = HashMap::new();
        let icp_addr = "rrkah-fqaaa-aaaaa-aaaaq-cai";
        patch_values.insert(
            RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL.to_string(),
            icp_addr.to_string(),
        );
        // enter blank value to remove the key
        patch_values.insert(RESOLVER_KEY_TWITTER.to_string(), "".to_string());
        let owner = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();



        let caller_registration = NAMED_CANISTER_IDS.with(|n| {
            let n = n.borrow();
            n.get_canister_id(CanisterNames::Registrar)
        });


        // add resolver
        add_test_resolver(name);

        let _ctx = mock_registry_api
            .expect_get_users()
            .returning(move |_name| {
                Ok(RegistryUsers {
                    owner,
                    operators: HashSet::new(),
                })
            });

        // act
        let mut context = SetRecordValueValidator::new(
            must_not_anonymous(&caller_registration).unwrap(),
            name.to_string(),
            patch_values,
            Resolver::new(name.to_string()),
        );
        context.registry_api = Arc::new(mock_registry_api);

        let result = context.validate().await;

        // assert
        assert!(result.is_ok(), "{:?}", result);
        let result = result.unwrap();
        assert_eq!(
            result.update_primary_name_input,
            UpdatePrimaryNameInput::Set(owner.clone())
        );
    }
    #[rstest]
    async fn test_set_record_validation_call_from_canister_not_registrar_permission_denied(_init_test: (),
                                                                        mut mock_registry_api: MockRegistryApi) {
        let name = "nice.ic";
        let mut patch_values: HashMap<String, String> = HashMap::new();
        let icp_addr = "rrkah-fqaaa-aaaaa-aaaaq-cai";
        patch_values.insert(
            RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL.to_string(),
            icp_addr.to_string(),
        );
        // enter blank value to remove the key
        patch_values.insert(RESOLVER_KEY_TWITTER.to_string(), "".to_string());
        let owner = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();



        let caller_registration = NAMED_CANISTER_IDS.with(|n| {
            let n = n.borrow();
            n.get_canister_id(CanisterNames::Registry)
        });


        // add resolver
        add_test_resolver(name);

        let _ctx = mock_registry_api
            .expect_get_users()
            .returning(move |_name| {
                Ok(RegistryUsers {
                    owner,
                    operators: HashSet::new(),
                })
            });

        // act
        let mut context = SetRecordValueValidator::new(
            must_not_anonymous(&caller_registration).unwrap(),
            name.to_string(),
            patch_values,
            Resolver::new(name.to_string()),
        );
        context.registry_api = Arc::new(mock_registry_api);

        let result = context.validate().await;

        // assert
        assert!(result.is_err());
        let result = result.err().unwrap();
        assert_eq!(result, NamingError::PermissionDenied);
    }
}

mod validate_value {
    use super::*;

    #[rstest]
    #[case("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa", true)]
    #[case("1A1zP1eP5QGefi2DMPTfTL5SLmv7Divf2a", false)]
    #[case("1A1zP1eP5QGefi2DMPTfTL5SLmv7Divf", false)]
    fn test_btc_valid_value(#[case] value: String, #[case] expected: bool) {
        let result = validate_well_known_value(&WellKnownResolverKey::Btc, &value);
        assert_eq!(expected, result.is_ok());
    }

    #[rstest]
    #[case("0xb436ef6cc9f24193ccb42f98be2b1db764484514", true)]
    #[case("0xb436ef6cc9f24193ccb42f98be2b1db7644845w4", false)]
    #[case("0xb436ef6cc9f24193ccb42f98be2b1db7644845", false)]
    fn test_eth_valid_value(#[case] value: String, #[case] expected: bool) {
        let result = validate_well_known_value(&WellKnownResolverKey::Eth, &value);
        assert_eq!(expected, result.is_ok());
    }

    #[rstest]
    #[case("LUwxSibYhxq2u6RfhQmkuTPZRk2wNjwLbE", true)]
    #[case("LMDPD5BLE2G7GGZbboAArBRSXvFBrTC12d", true)]
    #[case("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa", false)]
    #[case("0XB436EF6CC9F24193CCB42F98BE2B1DB7644845", false)]
    fn test_ltc_valid_value(#[case] value: String, #[case] expected: bool) {
        let result = validate_well_known_value(&WellKnownResolverKey::Ltc, &value);
        assert_eq!(expected, result.is_ok());
    }

    #[rstest]
    #[case(
    "cc659fe529756bae6f72db9937c6c60cf7ad57eb4ac5f930a75748927aab469a",
    true
    )]
    #[case(
    "92dd9b9ad9c5e937aaf0136a5ec313f6f86aeab08951e52a92b4bb5f3b6017f4",
    true
    )]
    #[case(
    "uqf5b-uk33j-b72z7-uoz2o-hmhl2-lw63v-zwh5f-cmnii-k4pzi-jbomw-nae",
    true
    )]
    #[case("q3fc5-haaaa-aaaaa-aaahq-cai", true)]
    #[case("aaaaa-aaaaa-aaaaa-aaahq-cai", false)]
    fn test_icp_valid_value(#[case] value: String, #[case] expected: bool) {
        let result = validate_well_known_value(&WellKnownResolverKey::Icp, &value);
        assert_eq!(expected, result.is_ok());
    }
}

mod remove_resolvers {
    use common::named_canister_ids::{get_named_get_canister_id, CanisterNames};

    use super::*;

    #[rstest]
    fn test_remove_resolvers_success(service: ResolverService, mock_now: u64) {
        STATE.with(|s| {
            let mut store = s.resolver_store.borrow_mut();
            store.ensure_created("test1.ic");
            store.ensure_created("test2.ic");
            store.ensure_created("app.test3.ic");
            store.ensure_created("app.nice.ic");
        });

        // act
        let caller = get_named_get_canister_id(CanisterNames::Registry);
        let names = vec!["app.test3.ic".to_string(), "test2.ic".to_string()];
        let call_context = CallContext::new(caller, TimeInNs(mock_now));
        let result = service.remove_resolvers(call_context, names);

        // assert
        assert!(result.is_ok());

        STATE.with(|s| {
            let store = s.resolver_store.borrow();
            let resolvers = store.get_resolvers();
            assert_eq!(resolvers.len(), 2);
            resolvers.get("test1.ic").unwrap();
            resolvers.get("app.nice.ic").unwrap();
        })
    }

    #[rstest]
    fn test_remove_resolvers_success_even_not_found(service: ResolverService, mock_now: u64) {
        // act
        let names = vec!["app.test3.ic".to_string(), "test2.ic".to_string()];
        let caller = get_named_get_canister_id(CanisterNames::Registry);
        let call_context = CallContext::new(caller, TimeInNs(mock_now));
        let result = service.remove_resolvers(call_context, names);

        // assert
        assert!(result.is_ok());
    }

    #[rstest]
    fn test_remove_resolvers_failed_not_admin(service: ResolverService) {
        // act
        let names = vec!["app.test3.ic".to_string(), "test2.ic".to_string()];
        let result = service.remove_resolvers(CallContext::anonymous(), names);

        // assert
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), NamingError::Unauthorized);
    }
}

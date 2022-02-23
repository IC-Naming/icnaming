use std::collections::HashSet;

use candid::Principal;
use rstest::*;

use common::constants::*;
use common::dto::RegistryUsers;
use test_common::canister_api::mock_registry_api;
use test_common::canister_api::MockRegistryApi;
use test_common::ic_api::init_test;
use test_common::ic_api::test_wrapper::set_caller;
use test_common::ic_api::test_wrapper::TestICApi;

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
    let service = ResolverService::new();
    service
}

mod ensure_resolver_created {
    use super::*;

    #[rstest]
    fn test_ensure_resolver_created(_init_test: (), mut service: ResolverService) {
        let name = "nice.icp";

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
        let name = "nice.icp";

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
        let name = "nice.icp";

        // act
        let result = service.get_record_value(name);

        // assert
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 0);
    }

    #[rstest]
    fn test_get_record_value_found(_init_test: (), service: ResolverService) {
        let name = "nice.icp";
        add_test_resolver(name);

        // act
        let result = service.get_record_value(name);

        // assert
        let map = result.unwrap();
        assert_eq!(map.len(), 2);
        assert!(map.contains_key(RESOLVER_KEY_GITHUB));
    }
}

mod set_record {
    use super::*;

    #[rstest]
    async fn test_set_record_value_key_invalid(_init_test: (), mut service: ResolverService) {
        let name = "nice.icp";
        let mut patch_values: HashMap<String, String> = HashMap::new();
        let invalid_resolver_key = "not_found";
        patch_values.insert(invalid_resolver_key.to_string(), "icns".to_string());
        // add resolver
        add_test_resolver(name);

        // act
        let result = service.set_record_value(name, patch_values).await;

        // assert
        assert!(result.is_err());
        match result {
            Err(e) => {
                assert_eq!(
                    e,
                    ICNSError::InvalidResolverKey {
                        key: invalid_resolver_key.to_string()
                    }
                );
            }
            _ => assert!(false),
        }
    }

    #[rstest]
    async fn test_set_record_value_value_invalid(_init_test: (), mut service: ResolverService) {
        let name = "nice.icp";
        let mut patch_values: HashMap<String, String> = HashMap::new();
        // create string value with 257 chars
        let mut value = String::new();
        for _ in 0..257 {
            value.push('a');
        }
        patch_values.insert(RESOLVER_KEY_GITHUB.to_string(), value);

        // add resolver
        add_test_resolver(name);

        // act
        let result = service.set_record_value(name, patch_values).await;

        // assert
        assert!(result.is_err());
        match result {
            Err(e) => {
                assert_eq!(
                    e,
                    ICNSError::ValueMaxLengthError {
                        max: RESOLVER_VALUE_MAX_LENGTH
                    }
                );
            }
            _ => assert!(false),
        }
    }

    #[rstest]
    async fn test_set_record_value_permission_deny(
        _init_test: (),
        mut service: ResolverService,
        mut mock_registry_api: MockRegistryApi,
    ) {
        let name = "nice.icp";
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
        service.registry_api = Arc::new(mock_registry_api);
        service.request_context = Arc::new(TestICApi {});
        set_caller(Principal::anonymous());

        // act
        let result = service.set_record_value(name, patch_values).await;

        // assert
        assert!(result.is_err());
        match result {
            Err(e) => {
                assert_eq!(e, ICNSError::PermissionDenied {});
            }
            _ => assert!(false),
        }
    }

    #[rstest]
    async fn test_set_record_value_success(
        _init_test: (),
        mut service: ResolverService,
        mut mock_registry_api: MockRegistryApi,
    ) {
        let name = "nice.icp";
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
        service.registry_api = Arc::new(mock_registry_api);
        service.request_context = Arc::new(TestICApi {});
        set_caller(owner);

        // act
        let result = service.set_record_value(name, patch_values).await;

        // assert
        assert!(result.is_ok());
        let value_map = service.get_record_value(name).unwrap();
        assert_eq!(value_map.len(), 2);
        assert_eq!(value_map.get(RESOLVER_KEY_ICP).unwrap(), &icp_addr);
        assert!(value_map.get(RESOLVER_KEY_TWITTER).is_none());
    }

    #[rstest]
    async fn test_set_record_value_not_found(
        _init_test: (),
        mut service: ResolverService,
        mut mock_registry_api: MockRegistryApi,
    ) {
        let name = "nice.icp";
        let mut patch_values: HashMap<String, String> = HashMap::new();
        patch_values.insert(RESOLVER_KEY_GITHUB.to_string(), "icns".to_string());
        let owner = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();

        let _ctx = mock_registry_api
            .expect_get_users()
            .returning(move |_name| {
                Ok(RegistryUsers {
                    owner,
                    operators: HashSet::new(),
                })
            });
        service.registry_api = Arc::new(mock_registry_api);
        service.request_context = Arc::new(TestICApi {});
        set_caller(owner);

        // act
        let result = service.set_record_value(name, patch_values).await;

        // assert
        assert!(result.is_ok());
        let value_map = service.get_record_value(name).unwrap();
        assert_eq!(value_map.len(), 1);
        assert_eq!(value_map.get(RESOLVER_KEY_GITHUB).unwrap(), "icns");
    }
}

mod validate_value {
    use super::*;

    #[rstest]
    #[case("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa", true)]
    #[case("1A1zP1eP5QGefi2DMPTfTL5SLmv7Divf2a", false)]
    #[case("1A1zP1eP5QGefi2DMPTfTL5SLmv7Divf", false)]
    fn test_btc_valid_value(#[case] value: String, #[case] expected: bool) {
        let result = validate_value(&ResolverKey::Btc, &value);
        assert_eq!(expected, result.is_ok());
    }

    #[rstest]
    #[case("0xb436ef6cc9f24193ccb42f98be2b1db764484514", true)]
    #[case("0xb436ef6cc9f24193ccb42f98be2b1db7644845w4", false)]
    #[case("0xb436ef6cc9f24193ccb42f98be2b1db7644845", false)]
    fn test_eth_valid_value(#[case] value: String, #[case] expected: bool) {
        let result = validate_value(&ResolverKey::Eth, &value);
        assert_eq!(expected, result.is_ok());
    }

    #[rstest]
    #[case("LUwxSibYhxq2u6RfhQmkuTPZRk2wNjwLbE", true)]
    #[case("LMDPD5BLE2G7GGZbboAArBRSXvFBrTC12d", true)]
    #[case("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa", false)]
    #[case("0XB436EF6CC9F24193CCB42F98BE2B1DB7644845", false)]
    fn test_ltc_valid_value(#[case] value: String, #[case] expected: bool) {
        let result = validate_value(&ResolverKey::Ltc, &value);
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
        let result = validate_value(&ResolverKey::Icp, &value);
        assert_eq!(expected, result.is_ok());
    }
}

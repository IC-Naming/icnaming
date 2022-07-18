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

mod validate_well_known_value {
    use super::*;

    #[rstest]
    #[case("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa", true)]
    #[case("1A1zP1eP5QGefi2DMPTfTL5SLmv7Divf2a", false)]
    #[case("1A1zP1eP5QGefi2DMPTfTL5SLmv7Divf", false)]
    fn test_btc_valid_value(#[case] value: String, #[case] expected: bool) {
        let result = validate_well_known_value(&ResolverKey::Btc, &value);
        assert_eq!(expected, result.is_ok());
    }

    #[rstest]
    #[case("0xb436ef6cc9f24193ccb42f98be2b1db764484514", true)]
    #[case("0xb436ef6cc9f24193ccb42f98be2b1db7644845w4", false)]
    #[case("0xb436ef6cc9f24193ccb42f98be2b1db7644845", false)]
    fn test_eth_valid_value(#[case] value: String, #[case] expected: bool) {
        let result = validate_well_known_value(&ResolverKey::Eth, &value);
        assert_eq!(expected, result.is_ok());
    }

    #[rstest]
    #[case("LUwxSibYhxq2u6RfhQmkuTPZRk2wNjwLbE", true)]
    #[case("LMDPD5BLE2G7GGZbboAArBRSXvFBrTC12d", true)]
    #[case("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa", false)]
    #[case("0XB436EF6CC9F24193CCB42F98BE2B1DB7644845", false)]
    fn test_ltc_valid_value(#[case] value: String, #[case] expected: bool) {
        let result = validate_well_known_value(&ResolverKey::Ltc, &value);
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
        let result = validate_well_known_value(&ResolverKey::Icp, &value);
        assert_eq!(expected, result.is_ok());
    }
}

mod remove_resolvers {
    use common::named_canister_ids::get_named_get_canister_id;

    use super::*;

    #[rstest]
    fn test_remove_resolvers_success(service: ResolverService) {
        STATE.with(|s| {
            let mut store = s.resolver_store.borrow_mut();
            store.ensure_created("test1.icp");
            store.ensure_created("test2.icp");
            store.ensure_created("app.test3.icp");
            store.ensure_created("app.nice.icp");
        });

        // act
        let caller = get_named_get_canister_id(CANISTER_NAME_REGISTRY);
        let names = vec!["app.test3.icp".to_string(), "test2.icp".to_string()];
        let result = service.remove_resolvers(&caller, names);

        // assert
        assert!(result.is_ok());

        STATE.with(|s| {
            let store = s.resolver_store.borrow();
            let resolvers = store.get_resolvers();
            assert_eq!(resolvers.len(), 2);
            resolvers.get("test1.icp").unwrap();
            resolvers.get("app.nice.icp").unwrap();
        })
    }

    #[rstest]
    fn test_remove_resolvers_success_even_not_found(service: ResolverService) {
        // act
        let caller = get_named_get_canister_id(CANISTER_NAME_REGISTRY);
        let names = vec!["app.test3.icp".to_string(), "test2.icp".to_string()];
        let result = service.remove_resolvers(&caller, names);

        // assert
        assert!(result.is_ok());
    }

    #[rstest]
    fn test_remove_resolvers_failed_not_admin(service: ResolverService) {
        // act
        let names = vec!["app.test3.icp".to_string(), "test2.icp".to_string()];
        let result = service.remove_resolvers(&Principal::anonymous(), names);

        // assert
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ICNSError::Unauthorized);
    }
}

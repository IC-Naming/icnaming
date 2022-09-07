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

fn add_test_resolver(name: &str) -> Resolver {
    STATE.with(|s| {
        let mut store = s.resolver_store.borrow_mut();
        let resolvers = store.get_resolvers_mut();
        let mut resolver = Resolver::new(name.to_string());
        resolver.set_record_value(RESOLVER_KEY_GITHUB.to_string(), "icns".to_string());
        resolver.set_record_value(RESOLVER_KEY_TWITTER.to_string(), "twitter".to_string());
        resolvers.insert(name.to_string(), resolver).unwrap()
    })
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

mod remove_resolvers {
    use common::named_canister_ids::{get_named_get_canister_id, CanisterNames};

    use super::*;

    #[rstest]
    fn test_remove_resolvers_success(
        service: ResolverService,
        mock_now: u64,
        mock_user1: Principal,
    ) {
        STATE.with(|s| {
            let mut store = s.resolver_store.borrow_mut();
            store.ensure_created("test1.ic");
            store.ensure_created("test2.ic");
            store.ensure_created("app.test3.ic");
            store.ensure_created("app.nice.ic");

            let mut store = s.reverse_resolver_store.borrow_mut();
            store.set_primary_name(mock_user1, "app.test3.ic".to_string());
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

            let store = s.reverse_resolver_store.borrow();
            assert_eq!(
                store.get_primary_name_reverse(&"app.test3.ic".to_string()),
                None
            );
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

mod batch_get_reverse_resolver {
    use super::*;

    #[rstest]
    fn test_batch_get_reverse_resolver_success(
        service: ResolverService,
        mock_user1: Principal,
        mock_user2: Principal,
    ) {
        let test1_str = "test1.ic";
        let test2_str = "test2.ic";
        STATE.with(|s| {
            let mut store = s.resolver_store.borrow_mut();
            store.ensure_created(test1_str.clone());
            store.ensure_created(test2_str.clone());

            let mut store = s.reverse_resolver_store.borrow_mut();
            store.set_primary_name(mock_user1.clone(), test1_str.into());
            store.set_primary_name(mock_user2.clone(), test2_str.into());
        });

        let principals = vec![mock_user1, mock_user2];

        // act
        let result = service.batch_get_reverse_resolve_principal(principals);

        //assert
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[rstest]
    fn test_batch_get_reverse_resolver_failed_anonymous(
        service: ResolverService,
        mock_user1: Principal,
        mock_user2: Principal,
    ) {
        let test1_str = "test1.ic";
        let test2_str = "test2.ic";
        let test3_str = "test3.ic";
        let anonymous = Principal::anonymous();
        STATE.with(|s| {
            let mut store = s.resolver_store.borrow_mut();
            store.ensure_created(test1_str.clone());
            store.ensure_created(test2_str.clone());

            let mut store = s.reverse_resolver_store.borrow_mut();
            store.set_primary_name(mock_user1.clone(), test1_str.into());
            store.set_primary_name(mock_user2.clone(), test2_str.into());
            store.set_primary_name(anonymous.clone(), test3_str.into());
        });

        let principals = vec![mock_user1, mock_user2, anonymous];

        // act
        let result = service.batch_get_reverse_resolve_principal(principals);

        //assert
        assert!(result.is_err());
    }
}

mod set_record_validation {
    use super::*;
    use crate::set_record_value_input::{UpdatePrimaryNameInput, UpdateRecordInput};
}
mod import_record_value {
    use super::*;
    use crate::set_record_value_input::{
        PatchValueOperation, UpdatePrimaryNameInput, UpdateRecordInput,
    };
    use common::permissions::get_admin;
    fn generate_resolver_value_import_item(
        name: &str,
        key: &str,
        value: &str,
        operator: String,
    ) -> ResolverValueImportItem {
        match operator.as_str() {
            "upsert" => {
                let item = ResolverValueImportItem {
                    name: name.to_string(),
                    key: key.to_string(),
                    value_and_operation: PatchValueOperation::Upsert(value.to_string()),
                };
                item
            }
            "insert_or_ignore" => {
                let item = ResolverValueImportItem {
                    name: name.to_string(),
                    key: key.to_string(),
                    value_and_operation: PatchValueOperation::InsertOrIgnore(value.to_string()),
                };
                item
            }
            "remove" => {
                let item = ResolverValueImportItem {
                    name: name.to_string(),
                    key: key.to_string(),
                    value_and_operation: PatchValueOperation::Remove,
                };
                item
            }
            _ => panic!("invalid operator"),
        }
    }

    fn get_expect_update_record_input(value: &str, operator: String) -> UpdateRecordInput {
        match operator.as_str() {
            "upsert" => UpdateRecordInput::Set(value.to_string()),
            "insert_or_ignore" => UpdateRecordInput::InsertOrIgnore(value.to_string()),
            "remove" => UpdateRecordInput::Remove,
            _ => panic!("invalid operator"),
        }
    }

    fn get_expect_update_primary_name_input(
        value: &str,
        operator: String,
    ) -> UpdatePrimaryNameInput {
        match operator.as_str() {
            "upsert" => UpdatePrimaryNameInput::Set(Principal::from_text(value).unwrap()),
            "insert_or_ignore" => {
                UpdatePrimaryNameInput::InsertOrIgnore(Principal::from_text(value).unwrap())
            }
            "remove" => UpdatePrimaryNameInput::Remove,
            _ => panic!("invalid operator"),
        }
    }

    #[rstest]
    fn test_import_record_value_insert_ignore_success(
        _init_test: (),
        service: ResolverService,
        _mock_now: u64,
    ) {
        // arrange
        let admin = get_admin();
        let call_context = CallContext::new(admin, TimeInNs(_mock_now));
        let name = "nice.ic";
        let icp_addr_before = "rrkah-fqaaa-aaaaa-aaaaq-cai";
        let icp_addr_after = "qjdve-lqaaa-aaaaa-aaaeq-cai";
        let before_item = generate_resolver_value_import_item(
            name,
            RESOLVER_KEY_ICP,
            icp_addr_before,
            "insert_or_ignore".to_string(),
        );
        let after_item = generate_resolver_value_import_item(
            name,
            RESOLVER_KEY_ICP,
            icp_addr_after,
            "insert_or_ignore".to_string(),
        );

        let list = vec![before_item, after_item];
        // add resolver
        add_test_resolver(name);

        // act
        let result = service.import_record_value(&call_context, list);
        let validation_result = service.get_record_value(name);

        // assert
        assert!(result.is_ok(), "{:?}", result);
        assert!(validation_result.is_ok(), "{:?}", validation_result);
        let validation_result = validation_result.unwrap();
        assert_eq!(
            validation_result.get(RESOLVER_KEY_ICP).unwrap(),
            icp_addr_before
        );
    }

    #[rstest]
    fn test_import_record_value_primary_name_insert_ignore_success(
        _init_test: (),
        service: ResolverService,
        _mock_now: u64,
    ) {
        // arrange
        let admin = get_admin();
        let call_context = CallContext::new(admin, TimeInNs(_mock_now));
        let name = "nice.ic";
        let icp_addr_before = "rrkah-fqaaa-aaaaa-aaaaq-cai";
        let icp_addr_after = "qjdve-lqaaa-aaaaa-aaaeq-cai";
        let before_item = generate_resolver_value_import_item(
            name,
            RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL,
            icp_addr_before,
            "insert_or_ignore".to_string(),
        );
        let after_item = generate_resolver_value_import_item(
            name,
            RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL,
            icp_addr_after,
            "insert_or_ignore".to_string(),
        );

        let list = vec![before_item, after_item];
        // add resolver
        add_test_resolver(name);

        // act
        let result = service.import_record_value(&call_context, list);
        let validation_result = service.get_record_value(name);

        // assert
        assert!(result.is_ok(), "{:?}", result);
        assert!(validation_result.is_ok(), "{:?}", validation_result);
        let validation_result = validation_result.unwrap();
        debug!("{:?}", validation_result);
        assert_eq!(
            validation_result
                .get(RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL)
                .unwrap(),
            icp_addr_before
        );
    }

    #[rstest]
    fn test_import_record_value_service_remove_primary_name_empty_success(
        _init_test: (),
        service: ResolverService,
        _mock_now: u64,
    ) {
        // arrange
        let admin = get_admin();
        let call_context = CallContext::new(admin, TimeInNs(_mock_now));
        let name = "nice.ic";
        let icp_addr = "rrkah-fqaaa-aaaaa-aaaaq-cai";
        let before_item = generate_resolver_value_import_item(
            name,
            RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL,
            icp_addr,
            "insert_or_ignore".to_string(),
        );
        let remove_item = generate_resolver_value_import_item(
            name,
            RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL,
            "",
            "remove".to_string(),
        );

        let list = vec![before_item, remove_item];
        // add resolver
        add_test_resolver(name);

        // act
        let result = service.import_record_value(&call_context, list);
        let validation_result = service.get_record_value(name);

        // assert
        assert!(result.is_ok(), "{:?}", result);
        assert!(validation_result.is_ok(), "{:?}", validation_result);
        let validation_result = validation_result.unwrap();
        debug!("{:?}", validation_result);
        assert_eq!(
            validation_result.get(RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL),
            None
        );
    }
}

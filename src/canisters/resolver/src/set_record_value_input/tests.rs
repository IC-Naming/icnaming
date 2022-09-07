use std::collections::HashSet;

use candid::Principal;

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
        resolvers.insert(name.to_string(), resolver.clone());
        resolver
    })
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

mod set_record_validation {
    use super::*;
    use common::named_canister_ids::NAMED_CANISTER_IDS;
    use common::permissions::must_not_anonymous;

    #[rstest]
    async fn test_set_record_validation_success(
        _init_test: (),
        mut mock_registry_api: MockRegistryApi,
        _mock_now: u64,
        _mock_user1: Principal,
    ) {
        let name = "nice.ic";
        let mut patch_values: HashMap<String, String> = HashMap::new();
        let icp_addr = "rrkah-fqaaa-aaaaa-aaaaq-cai";
        patch_values.insert(RESOLVER_KEY_ICP.to_string(), icp_addr.to_string());
        // enter blank value to remove the key
        patch_values.insert(RESOLVER_KEY_TWITTER.to_string(), "".to_string());
        let owner = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();

        // add resolver
        let resolver = add_test_resolver(name);

        let _ctx = mock_registry_api
            .expect_get_users()
            .returning(move |_name| {
                Ok(RegistryUsers {
                    owner,
                    operators: HashSet::new(),
                })
            });

        // act
        let patch_values: PatchValuesInput = patch_values.into();
        let patch_value_validator =
            PatchValuesValidator::new(name.to_string(), patch_values, resolver);
        let mut owner_validator = patch_value_validator
            .validate_and_generate_owner_validator(must_not_anonymous(&owner).unwrap())
            .unwrap();
        owner_validator.registry_api = Arc::new(mock_registry_api);
        let input_generator = owner_validator.validate().await.unwrap();
        let result = input_generator.generate();

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
        mut mock_registry_api: MockRegistryApi,
        _mock_now: u64,
        _mock_user1: Principal,
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
        let resolver = add_test_resolver(name);

        let _ctx = mock_registry_api
            .expect_get_users()
            .returning(move |_name| {
                Ok(RegistryUsers {
                    owner,
                    operators: HashSet::new(),
                })
            });

        // act
        let patch_values: PatchValuesInput = patch_values.into();
        let patch_value_validator =
            PatchValuesValidator::new(name.to_string(), patch_values, resolver);

        let mut owner_validator = patch_value_validator
            .validate_and_generate_owner_validator(must_not_anonymous(&owner).unwrap())
            .unwrap();
        owner_validator.registry_api = Arc::new(mock_registry_api);
        let input_generator = owner_validator.validate().await.unwrap();
        let result = input_generator.generate();

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
        mut mock_registry_api: MockRegistryApi,
        _mock_now: u64,
        _mock_user1: Principal,
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
        let resolver = add_test_resolver(name);

        let _ctx = mock_registry_api
            .expect_get_users()
            .returning(move |_name| {
                Ok(RegistryUsers {
                    owner,
                    operators: HashSet::new(),
                })
            });

        // act
        let patch_values: PatchValuesInput = patch_values.into();
        let patch_value_validator =
            PatchValuesValidator::new(name.to_string(), patch_values, resolver);

        let mut owner_validator = patch_value_validator
            .validate_and_generate_owner_validator(must_not_anonymous(&owner).unwrap())
            .unwrap();
        owner_validator.registry_api = Arc::new(mock_registry_api);
        let input_generator = owner_validator.validate().await.unwrap();
        let result = input_generator.generate();

        // assert
        assert!(result.is_ok(), "{:?}", result);
        let result = result.unwrap();
        assert_eq!(
            result.update_primary_name_input,
            UpdatePrimaryNameInput::Remove
        );
    }

    #[rstest]
    async fn test_set_record_validation_key_too_long(
        _init_test: (),
        _mock_now: u64,
        mock_user1: Principal,
    ) {
        let name = "nice.ic";
        let mut patch_values: HashMap<String, String> = HashMap::new();
        let invalid_resolver_key = "a".repeat(RESOLVER_KEY_MAX_LENGTH + 1);
        patch_values.insert(invalid_resolver_key.to_string(), "icns".to_string());
        // add resolver
        let resolver = add_test_resolver(name);
        // act
        let patch_values: PatchValuesInput = patch_values.into();
        let patch_value_validator =
            PatchValuesValidator::new(name.to_string(), patch_values, resolver);
        let owner_validator = patch_value_validator
            .validate_and_generate_owner_validator(must_not_anonymous(&mock_user1).unwrap());

        // assert
        assert!(owner_validator.is_err());
        match owner_validator {
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
        _mock_now: u64,
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
        let resolver = add_test_resolver(name);

        // act
        let patch_values: PatchValuesInput = patch_values.into();
        let patch_value_validator =
            PatchValuesValidator::new(name.to_string(), patch_values, resolver);
        let owner_validator = patch_value_validator
            .validate_and_generate_owner_validator(must_not_anonymous(&mock_user1).unwrap());

        // assert
        assert!(owner_validator.is_err());
        match owner_validator {
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
        _mock_now: u64,
        mock_user1: Principal,
    ) {
        let name = "nice.ic";
        let mut patch_values: HashMap<String, String> = HashMap::new();
        patch_values.insert(RESOLVER_KEY_GITHUB.to_string(), "icns".to_string());
        // add resolver
        let mut resolver = add_test_resolver(name);

        // act
        for i in 0..RESOLVER_ITEM_MAX_COUNT {
            resolver.set_record_value(format!("{}", i), format!("{}", i));
        }
        let patch_values: PatchValuesInput = patch_values.into();
        let patch_value_validator =
            PatchValuesValidator::new(name.to_string(), patch_values, resolver.clone());
        let owner_validator = patch_value_validator
            .validate_and_generate_owner_validator(must_not_anonymous(&mock_user1).unwrap());

        // assert
        assert!(owner_validator.is_err());
        match owner_validator {
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
        _mock_now: u64,
        mock_user1: Principal,
    ) {
        let name = "nice.ic";
        let mut patch_values: HashMap<String, String> = HashMap::new();
        patch_values.insert(RESOLVER_KEY_GITHUB.to_string(), "icns".to_string());
        let owner = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();

        // add resolver
        let resolver = add_test_resolver(name);

        let _ctx = mock_registry_api
            .expect_get_users()
            .returning(move |_name| {
                Ok(RegistryUsers {
                    owner,
                    operators: HashSet::new(),
                })
            });

        // act
        let patch_values: PatchValuesInput = patch_values.into();
        let patch_value_validator =
            PatchValuesValidator::new(name.to_string(), patch_values, resolver);

        let mut owner_validator = patch_value_validator
            .validate_and_generate_owner_validator(must_not_anonymous(&mock_user1).unwrap())
            .unwrap();
        owner_validator.registry_api = Arc::new(mock_registry_api);
        let result = owner_validator.validate().await;

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
        let resolver = add_test_resolver(name);

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
        let patch_values: PatchValuesInput = patch_values.into();
        let patch_value_validator =
            PatchValuesValidator::new(name.to_string(), patch_values, resolver);

        let mut owner_validator = patch_value_validator
            .validate_and_generate_owner_validator(must_not_anonymous(&mock_user1).unwrap())
            .unwrap();
        owner_validator.registry_api = Arc::new(mock_registry_api);
        let input_generator = owner_validator.validate().await;

        // assert
        assert!(input_generator.is_err());
        let result = input_generator.err().unwrap();
        assert_eq!(result, NamingError::PermissionDenied);
    }

    #[rstest]
    async fn test_set_record_validation_call_from_canister_registrar_should_pass(
        _init_test: (),
        mut mock_registry_api: MockRegistryApi,
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

        let caller_registration = NAMED_CANISTER_IDS.with(|n| {
            let n = n.borrow();
            n.get_canister_id(CanisterNames::Registrar)
        });

        // add resolver
        let resolver = add_test_resolver(name);

        let _ctx = mock_registry_api
            .expect_get_users()
            .returning(move |_name| {
                Ok(RegistryUsers {
                    owner,
                    operators: HashSet::new(),
                })
            });

        // act
        let patch_values: PatchValuesInput = patch_values.into();
        let patch_value_validator =
            PatchValuesValidator::new(name.to_string(), patch_values, resolver);
        let mut owner_validator = patch_value_validator
            .validate_and_generate_owner_validator(
                must_not_anonymous(&caller_registration).unwrap(),
            )
            .unwrap();
        owner_validator.registry_api = Arc::new(mock_registry_api);
        let input_generator = owner_validator.validate().await.unwrap();
        let result = input_generator.generate();

        // assert
        assert!(result.is_ok(), "{:?}", result);
        let result = result.unwrap();
        assert_eq!(
            result.update_primary_name_input,
            UpdatePrimaryNameInput::Set(owner.clone())
        );
    }

    #[rstest]
    async fn test_set_record_validation_call_from_canister_not_registrar_permission_denied(
        _init_test: (),
        mut mock_registry_api: MockRegistryApi,
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

        let caller_registration = NAMED_CANISTER_IDS.with(|n| {
            let n = n.borrow();
            n.get_canister_id(CanisterNames::Registry)
        });

        // add resolver
        let resolver = add_test_resolver(name);

        let _ctx = mock_registry_api
            .expect_get_users()
            .returning(move |_name| {
                Ok(RegistryUsers {
                    owner,
                    operators: HashSet::new(),
                })
            });

        // act
        let patch_values: PatchValuesInput = patch_values.into();
        let patch_value_validator =
            PatchValuesValidator::new(name.to_string(), patch_values, resolver);
        let mut owner_validator = patch_value_validator
            .validate_and_generate_owner_validator(
                must_not_anonymous(&caller_registration).unwrap(),
            )
            .unwrap();
        owner_validator.registry_api = Arc::new(mock_registry_api);
        let input_generator = owner_validator.validate().await;

        // assert
        assert!(input_generator.is_err());
        let result = input_generator.err().unwrap();
        assert_eq!(result, NamingError::PermissionDenied);
    }
}

mod import_record_value {
    use super::*;
    use crate::set_record_value_input::{
        PatchValueOperation, UpdatePrimaryNameInput, UpdateRecordInput,
    };
    use crate::ImportRecordValueRequest;

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
    #[case("upsert")]
    #[case("insert_or_ignore")]
    #[case("remove")]
    fn test_import_record_value_success(_init_test: (), #[case] operator: String) {
        // arrange
        let name = "nice.ic";
        let icp_addr = "rrkah-fqaaa-aaaaa-aaaaq-cai";
        let item =
            generate_resolver_value_import_item(name, RESOLVER_KEY_ICP, icp_addr, operator.clone());

        let expect_update_record_input = get_expect_update_record_input(icp_addr, operator.clone());

        // add resolver
        let resolver = add_test_resolver(name);

        // act
        let patch_values = item.into();
        let patch_values_validator: PatchValuesValidator =
            PatchValuesValidator::new(name.to_string(), patch_values, resolver);
        let input_generator = patch_values_validator
            .validate_and_generate_input_generator()
            .unwrap();
        let result = input_generator.generate();

        // assert
        assert!(result.is_ok(), "{:?}", result);
        let result = result.unwrap();
        assert_eq!(
            result.update_primary_name_input,
            UpdatePrimaryNameInput::DoNothing
        );
        assert_eq!(result.update_records_input.len(), 1);
        assert_eq!(
            result.update_records_input.get(RESOLVER_KEY_ICP).unwrap(),
            &expect_update_record_input
        );
    }

    #[rstest]
    #[case("upsert")]
    #[case("insert_or_ignore")]
    #[case("remove")]
    fn test_import_record_value_unknown_key_success(_init_test: (), #[case] operator: String) {
        // arrange
        let name = "nice.ic";
        let unknown_key = "@#$%^!";
        let icp_addr = "rrkah-fqaaa-aaaaa-aaaaq-cai";
        let item =
            generate_resolver_value_import_item(name, unknown_key, icp_addr, operator.clone());

        let expect_update_record_input = get_expect_update_record_input(icp_addr, operator.clone());

        // add resolver
        let resolver = add_test_resolver(name);

        // act
        let patch_values = item.into();
        let patch_values_validator: PatchValuesValidator =
            PatchValuesValidator::new(name.to_string(), patch_values, resolver);
        let input_generator = patch_values_validator
            .validate_and_generate_input_generator()
            .unwrap();
        let result = input_generator.generate();

        // assert
        assert!(result.is_ok(), "{:?}", result);
        let result = result.unwrap();
        assert_eq!(
            result.update_primary_name_input,
            UpdatePrimaryNameInput::DoNothing
        );
        assert_eq!(result.update_records_input.len(), 1);
        assert_eq!(
            result.update_records_input.get(unknown_key).unwrap(),
            &expect_update_record_input
        );
    }

    #[rstest]
    #[case("upsert")]
    #[case("insert_or_ignore")]
    #[case("remove")]
    fn test_import_record_value_primary_name_success(_init_test: (), #[case] operator: String) {
        // arrange
        let name = "nice.ic";
        let icp_addr = "rrkah-fqaaa-aaaaa-aaaaq-cai";
        let item = generate_resolver_value_import_item(
            name,
            RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL,
            icp_addr,
            operator.clone(),
        );

        let expect_update_primary_name_input =
            get_expect_update_primary_name_input(icp_addr, operator.clone());

        // add resolver
        let resolver = add_test_resolver(name);

        // act
        let patch_values = item.into();
        let patch_values_validator: PatchValuesValidator =
            PatchValuesValidator::new(name.to_string(), patch_values, resolver);
        let input_generator = patch_values_validator
            .validate_and_generate_input_generator()
            .unwrap();
        let result = input_generator.generate();

        // assert
        assert!(result.is_ok(), "{:?}", result);
        let result = result.unwrap();
        assert_eq!(
            result.update_primary_name_input,
            expect_update_primary_name_input
        );
        assert!(result.update_records_input.is_empty());
    }

    #[rstest]
    fn test_import_record_value_validator_remove_primary_name_empty_success(_init_test: ()) {
        // arrange
        let name = "nice.ic";
        let item = generate_resolver_value_import_item(
            name,
            RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL,
            "",
            "remove".to_string(),
        );

        // add resolver
        let resolver = add_test_resolver(name);

        // act
        let patch_values = item.into();
        let patch_values_validator: PatchValuesValidator =
            PatchValuesValidator::new(name.to_string(), patch_values, resolver);
        let input_generator = patch_values_validator
            .validate_and_generate_input_generator()
            .unwrap();
        let result = input_generator.generate();

        // assert
        assert!(result.is_ok(), "{:?}", result);
        let result = result.unwrap();
        assert!(result.update_records_input.is_empty());
        assert_eq!(
            result.update_primary_name_input,
            UpdatePrimaryNameInput::Remove
        );
    }

    #[rstest]
    #[case("upsert")]
    #[case("insert_or_ignore")]
    fn test_import_record_value_validator_upsert_or_insert_value_too_short(
        _init_test: (),
        #[case] operator: String,
    ) {
        // arrange
        let name = "nice.ic";
        let item =
            generate_resolver_value_import_item(name, RESOLVER_KEY_ICP, "", operator.to_string());
        let expect_update_record_input = get_expect_update_record_input("", operator.to_string());

        // add resolver
        let resolver = add_test_resolver(name);

        // act
        let patch_values = item.into();
        let patch_values_validator: PatchValuesValidator =
            PatchValuesValidator::new(name.to_string(), patch_values, resolver);
        let input_generator = patch_values_validator
            .validate_and_generate_input_generator()
            .unwrap();
        let result = input_generator.generate();

        // assert
        assert!(result.is_ok(), "{:?}", result);
        let result = result.unwrap();
        assert_eq!(
            result.update_records_input.get(RESOLVER_KEY_ICP).unwrap(),
            &expect_update_record_input
        );
    }

    #[rstest]
    fn test_import_record_value_remove_primary_name_normal_success(_init_test: ()) {
        // arrange
        let name = "nice.ic";
        let item = generate_resolver_value_import_item(
            name,
            RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL,
            "",
            "remove".to_string(),
        );

        // add resolver
        let resolver = add_test_resolver(name);

        // act
        let patch_values = item.into();
        let patch_values_validator: PatchValuesValidator =
            PatchValuesValidator::new(name.to_string(), patch_values, resolver);
        let input_generator = patch_values_validator
            .validate_and_generate_input_generator()
            .unwrap();
        let result = input_generator.generate();

        // assert
        assert!(result.is_ok(), "{:?}", result);
        let result = result.unwrap();
        assert_eq!(
            result.update_primary_name_input,
            UpdatePrimaryNameInput::Remove
        );
        assert!(result.update_records_input.is_empty());
    }

    #[rstest]
    fn test_import_record_value_upsert_primary_name_addr_invalid_principal(_init_test: ()) {
        // arrange
        let name = "nice.ic";
        let primary_name_addr = "123456789";
        let item = generate_resolver_value_import_item(
            name,
            RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL,
            primary_name_addr,
            "upsert".to_string(),
        );

        // add resolver
        let resolver = add_test_resolver(name);

        // act
        let patch_values = item.into();
        let patch_values_validator: PatchValuesValidator =
            PatchValuesValidator::new(name.to_string(), patch_values, resolver);
        let input_generator = patch_values_validator.validate_and_generate_input_generator();
        // assert
        assert!(input_generator.is_err(), "{:?}", input_generator);
        assert_eq!(
            input_generator.unwrap_err(),
            NamingError::InvalidResolverValueFormat {
                value: primary_name_addr.to_string(),
                format: "it is no a valid principal text".to_string(),
            }
        );
    }

    #[rstest]
    #[case("upsert")]
    #[case("insert_or_ignore")]
    fn test_import_record_value_validation_value_invalid(_init_test: (), #[case] operator: String) {
        // arrange
        let name = "nice.ic";
        let mut value = String::new();
        for _ in 0..(RESOLVER_VALUE_MAX_LENGTH + 1) {
            value.push('a');
        }
        let item =
            generate_resolver_value_import_item(name, RESOLVER_KEY_ICP, value.as_str(), operator);

        // add resolver
        let resolver = add_test_resolver(name);

        // act
        let patch_values = item.into();
        let patch_values_validator: PatchValuesValidator =
            PatchValuesValidator::new(name.to_string(), patch_values, resolver);
        let input_generator = patch_values_validator.validate_and_generate_input_generator();

        // assert
        assert!(input_generator.is_err());
        match input_generator {
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
    #[case("upsert")]
    #[case("insert_or_ignore")]
    fn test_import_record_value_primary_name_value_validation_value_invalid(
        _init_test: (),
        #[case] operator: String,
    ) {
        // arrange
        let name = "nice.ic";
        let mut value = String::new();
        for _ in 0..(RESOLVER_VALUE_MAX_LENGTH + 1) {
            value.push('a');
        }
        let item = generate_resolver_value_import_item(
            name,
            RESOLVER_KEY_SETTING_REVERSE_RESOLUTION_PRINCIPAL,
            value.as_str(),
            operator,
        );

        // add resolver
        let resolver = add_test_resolver(name);

        // act
        let patch_values = item.into();
        let patch_values_validator: PatchValuesValidator =
            PatchValuesValidator::new(name.to_string(), patch_values, resolver);
        let input_generator = patch_values_validator.validate_and_generate_input_generator();
        // assert
        assert!(input_generator.is_err());
        match input_generator {
            Err(e) => {
                assert_eq!(
                    e,
                    NamingError::InvalidResolverValueFormat {
                        value: value.to_string(),
                        format: "it is no a valid principal text".to_string(),
                    }
                );
            }
            _ => assert!(false),
        }
    }

    #[rstest]
    #[case("upsert")]
    #[case("insert_or_ignore")]
    #[case("remove")]
    fn test_set_record_validation_key_too_long(_init_test: (), #[case] operator: String) {
        // arrange
        let name = "nice.ic";
        let invalid_resolver_key = "a".repeat(RESOLVER_KEY_MAX_LENGTH + 1);
        let value = "icns".to_string();
        let item = generate_resolver_value_import_item(
            name,
            invalid_resolver_key.as_str(),
            value.as_str(),
            operator,
        );

        // add resolver
        let resolver = add_test_resolver(name);

        // act
        let patch_values = item.into();
        let patch_values_validator: PatchValuesValidator =
            PatchValuesValidator::new(name.to_string(), patch_values, resolver);
        let input_generator = patch_values_validator.validate_and_generate_input_generator();

        // assert
        assert!(input_generator.is_err());
        match input_generator {
            Err(e) => {
                assert_eq!(
                    e,
                    NamingError::KeyMaxLengthError {
                        max: RESOLVER_KEY_MAX_LENGTH
                    }
                );
            }
            _ => assert!(false),
        }
    }

    #[rstest]
    fn group_up_resolver_value_import_items_test(_init_test: ()) {
        let key_list = vec![RESOLVER_KEY_ICP, RESOLVER_KEY_BTC, RESOLVER_KEY_ETH];
        let operation_list = vec!["upsert", "upsert", "insert_or_ignore", "remove", "upsert"];
        let mut list = vec![];
        let nice = "nice.ic";
        let hello = "hello.ic";

        for key in &key_list {
            for operation in &operation_list {
                let item =
                    generate_resolver_value_import_item(nice, key, "value", operation.to_string());
                list.push(item);
            }
        }
        for key in &key_list {
            for operation in &operation_list {
                let item =
                    generate_resolver_value_import_item(hello, key, "value", operation.to_string());
                list.push(item);
            }
        }
        let request = ImportRecordValueRequest { items: list };

        let result = request.group_up_resolver_value_import_items();

        for item in result.clone() {
            debug!("name: {}", item.name);
            for sub_item in item.patch_values {
                debug!("{:?}", sub_item);
            }
        }

        let nice_group = result.iter().find(|item| item.name == nice).unwrap();

        assert_eq!(nice_group.patch_values.len(), operation_list.len());
        for (i, operation) in operation_list.iter().enumerate() {
            for key in &key_list {
                let item = nice_group.patch_values[i].0.get(key.to_owned()).unwrap();
                let expect_item =
                    generate_resolver_value_import_item(nice, key, "value", operation.to_string());
                assert_eq!(item.to_owned(), expect_item.value_and_operation);
            }
        }

        let hello_group = result.iter().find(|item| item.name == hello).unwrap();
        assert_eq!(hello_group.patch_values.len(), operation_list.len());
        for (i, operation) in operation_list.iter().enumerate() {
            for key in &key_list {
                let item = hello_group.patch_values[i].0.get(key.to_owned()).unwrap();
                let expect_item =
                    generate_resolver_value_import_item(hello, key, "value", operation.to_string());
                assert_eq!(item.to_owned(), expect_item.value_and_operation);
            }
        }
    }
}

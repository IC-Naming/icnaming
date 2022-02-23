use candid::Principal;
use rstest::*;

use common::constants::{DEFAULT_TTL, TOP_LABEL};
use common::errors::ICNSError;
use test_common::canister_api::*;
use test_common::ic_api::init_test;

use super::*;

#[fixture]
fn top_owner() -> Principal {
    Principal::from_text("xzrh4-zyaaa-aaaaj-qagaa-cai").unwrap()
}

#[fixture]
fn auth_caller() -> Principal {
    Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap()
}

#[fixture]
fn service() -> RegistriesService {
    RegistriesService::new()
}

pub(crate) fn create_registry(_name: String, _owner: Principal) -> Registry {
    Registry::new(
        TOP_LABEL.to_string(),
        top_owner(),
        DEFAULT_TTL,
        Principal::anonymous(),
    )
}

#[fixture]
fn add_test_registry() -> Registry {
    STATE.with(|s| {
        let mut store = s.registry_store.borrow_mut();
        let registries = store.get_registries_mut();
        let registry = create_registry("icp".to_string(), top_owner());
        registries.insert("icp".to_string(), registry.clone());
        registry
    })
}

mod add_top_name {
    use super::*;

    #[rstest]
    fn test_add_top_name(_init_test: (), mut service: RegistriesService, top_owner: Principal) {
        let icp_registry = create_registry(TOP_LABEL.to_string(), top_owner);
        let result = service.set_top_name(icp_registry);
        assert!(result.is_ok());
        let icp_registry = create_registry(TOP_LABEL.to_string(), top_owner);
        let result = service.set_top_name(icp_registry).err().unwrap();
        assert_eq!(result, ICNSError::TopNameAlreadyExists);

        STATE.with(|s| {
            let store = s.registry_store.borrow();
            let registries = store.get_registries();
            assert_eq!(registries.len(), 1);
            let item = get_registry(&registries, &TOP_LABEL.to_string()).unwrap();
            info!("{:?}", item);
            assert_eq!(item.get_name(), TOP_LABEL.to_string());
            assert_eq!(item.get_owner(), &top_owner);
            assert_eq!(item.get_ttl(), DEFAULT_TTL);
            assert_eq!(item.get_resolver(), Principal::anonymous());
        });
    }
}

mod add_subdomain_to_registries {
    use super::*;

    #[rstest]
    async fn test_add_subdomain_to_registries(
        _init_test: (),
        top_owner: Principal,
        mut service: RegistriesService,
        mut mock_resolver_api: MockResolverApi,
    ) {
        let _ctx = mock_resolver_api
            .expect_ensure_resolver_created()
            .returning(|_name| Ok(true));
        service.resolver_api = Arc::new(mock_resolver_api);
        service
            .set_top_name(create_registry(TOP_LABEL.to_string(), top_owner))
            .unwrap();

        let sub_owner = Principal::from_text("2vxsx-fae").unwrap();
        let result = service
            .set_subdomain_owner(
                "test".to_string(),
                "icp".to_string(),
                top_owner,
                sub_owner,
                128,
                Principal::anonymous(),
            )
            .await;
        println!("{:?}", result);
        assert!(result.is_ok());
        assert!(service.check_exist("test.icp".to_string()));

        STATE.with(|s| {
            let store = s.registry_store.borrow();
            let registries = store.get_registries();
            assert_eq!(registries.len(), 2);
            let item = get_registry(&registries, &"test.icp".to_string()).unwrap();
            info!("{:?}", item);
            assert_eq!(item.get_name(), "test.icp".to_string());
            assert_eq!(item.get_owner(), &sub_owner);
            assert_eq!(item.get_ttl(), 128);
            assert_eq!(item.get_resolver(), Principal::anonymous());
        });
    }
}

mod approval {
    use super::*;

    #[rstest]
    fn test_set_approval(
        _init_test: (),
        mut service: RegistriesService,
        add_test_registry: Registry,
    ) {
        let registry = add_test_registry;
        let operator = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
        // act
        let result = service.set_approval(registry.get_name(), registry.get_owner(), &operator);

        // assert
        assert!(result.is_ok());
        STATE.with(|s| {
            let store = s.registry_store.borrow();
            let registries = store.get_registries();
            let item = get_registry(&registries, &registry.get_name()).unwrap();
            let operators = item.get_operators().unwrap();
            assert_eq!(operators.len(), 1);
            assert!(operators.contains(&operator))
        });
    }

    #[rstest]
    fn test_set_approval_permission_deny(
        _init_test: (),
        mut service: RegistriesService,
        add_test_registry: Registry,
    ) {
        let registry = add_test_registry;
        let operator = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
        // act
        let result = service.set_approval(registry.get_name(), &Principal::anonymous(), &operator);

        // assert
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), ICNSError::PermissionDenied);
    }

    #[rstest]
    fn test_remove_approval(
        _init_test: (),
        mut service: RegistriesService,
        add_test_registry: Registry,
    ) {
        let registry = add_test_registry;
        let operator = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
        service
            .set_approval(registry.get_name(), registry.get_owner(), &operator)
            .unwrap();

        // act
        let result = service.remove_approval(registry.get_name(), registry.get_owner(), &operator);

        // assert
        assert!(result.is_ok());
        STATE.with(|s| {
            let store = s.registry_store.borrow();
            let registries = store.get_registries();
            let item = get_registry(&registries, &registry.get_name()).unwrap();
            let operators = item.get_operators().unwrap();
            assert_eq!(operators.len(), 0);
        });
    }

    #[rstest]
    fn test_remove_approval_permission_deny(_init_test: (), mut service: RegistriesService) {
        let registry = add_test_registry();
        let operator = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
        service
            .set_approval(registry.get_name(), registry.get_owner(), &operator)
            .unwrap();

        // act
        let result =
            service.remove_approval(registry.get_name(), &Principal::anonymous(), &operator);

        // assert
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), ICNSError::PermissionDenied);
    }
}

mod set_record {
    use super::*;

    #[rstest]
    fn test_set_record(_init_test: (), mut service: RegistriesService) {
        let caller = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
        let resolver = Principal::from_text("xzrh4-zyaaa-aaaaj-qagaa-cai").unwrap();
        let name = "icp";
        let ttl = 123;
        STATE.with(|s| {
            let mut store = s.registry_store.borrow_mut();
            let registries = store.get_registries_mut();
            let registry =
                Registry::new(name.to_string(), caller.clone(), 0, Principal::anonymous());
            registries.insert(name.to_string(), registry);
        });

        // act
        let result = service.set_record(&caller, name, ttl, &resolver);

        // assert
        assert!(result.is_ok());
        STATE.with(|s| {
            let store = s.registry_store.borrow();
            let registries = store.get_registries();
            let item = get_registry(&registries, &name.to_string()).unwrap();
            assert_eq!(item.get_name(), name.to_string());
            assert_eq!(item.get_ttl(), ttl);
            assert_eq!(item.get_resolver(), resolver);
        });
    }

    #[rstest]
    fn test_set_record_resolver_not_found(_init_test: (), mut service: RegistriesService) {
        let caller = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
        let resolver = Principal::from_text("xzrh4-zyaaa-aaaaj-qagaa-cai").unwrap();
        let name = "icp";
        let ttl = 123;

        // act
        let result = service.set_record(&caller, name, ttl, &resolver);

        // assert
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            ICNSError::RegistryNotFoundError {
                name: name.to_string()
            }
        );
    }

    #[rstest]
    fn test_set_record_permission_deny(_init_test: (), mut service: RegistriesService) {
        let _owner = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
        let resolver = Principal::from_text("xzrh4-zyaaa-aaaaj-qagaa-cai").unwrap();
        let name = "icp";
        let ttl = 123;
        STATE.with(|s| {
            let mut store = s.registry_store.borrow_mut();
            let registries = store.get_registries_mut();
            let registry = Registry::new(
                name.to_string(),
                resolver.clone(),
                0,
                Principal::anonymous(),
            );
            registries.insert(name.to_string(), registry);
        });

        // act
        let result = service.set_record(&Principal::anonymous(), name, ttl, &resolver);

        // assert
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), ICNSError::PermissionDenied);
    }
}

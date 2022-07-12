use candid::Principal;
use rstest::*;

use common::constants::{DEFAULT_TTL, NAMING_TOP_LABEL};
use common::errors::NamingError;
use common::named_canister_ids::{get_named_get_canister_id, CanisterNames};
use test_common::canister_api::*;
use test_common::ic_api::init_test;
use test_common::user::*;

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

#[fixture]
fn resolver() -> Principal {
    get_named_get_canister_id(CanisterNames::Resolver)
}

pub(crate) fn create_registry(_name: String, _owner: Principal) -> Registry {
    Registry::new(
        NAMING_TOP_LABEL.to_string(),
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
        let registry = create_registry(NAMING_TOP_LABEL.to_string(), top_owner());
        registries.insert(NAMING_TOP_LABEL.to_string(), registry.clone());
        registry
    })
}

mod add_top_name {
    use super::*;

    #[rstest]
    fn test_add_top_name(_init_test: (), mut service: RegistriesService, top_owner: Principal) {
        let icp_registry = create_registry(NAMING_TOP_LABEL.to_string(), top_owner);
        let result = service.set_top_name(icp_registry);
        assert!(result.is_ok());
        let icp_registry = create_registry(NAMING_TOP_LABEL.to_string(), top_owner);
        let result = service.set_top_name(icp_registry).err().unwrap();
        assert_eq!(result, NamingError::TopNameAlreadyExists);

        STATE.with(|s| {
            let store = s.registry_store.borrow();
            let registries = store.get_registries();
            assert_eq!(registries.len(), 1);
            let item = get_registry(&registries, &NAMING_TOP_LABEL.to_string()).unwrap();
            info!("{:?}", item);
            assert_eq!(item.get_name(), NAMING_TOP_LABEL.to_string());
            assert_eq!(item.get_owner(), &top_owner);
            assert_eq!(item.get_ttl(), DEFAULT_TTL);
            assert_eq!(item.get_resolver(), Principal::anonymous());
        });
    }
}

mod add_subdomain_to_registries {
    use super::*;
    use test_common::create_test_name;

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
            .set_top_name(create_registry(NAMING_TOP_LABEL.to_string(), top_owner))
            .unwrap();

        let sub_owner = Principal::from_text("2vxsx-fae").unwrap();
        let result = service
            .set_subdomain_owner(
                "test".to_string(),
                NAMING_TOP_LABEL.to_string(),
                top_owner,
                sub_owner,
                128,
                Principal::anonymous(),
            )
            .await;
        println!("{:?}", result);
        assert!(result.is_ok());
        let name = create_test_name("test");
        assert!(service.check_exist(&name));

        STATE.with(|s| {
            let store = s.registry_store.borrow();
            let registries = store.get_registries();
            assert_eq!(registries.len(), 2);
            let item = get_registry(&registries, &name).unwrap();
            info!("{:?}", item);
            assert_eq!(item.get_name(), name);
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
        assert_eq!(result.err().unwrap(), NamingError::PermissionDenied);
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
        assert_eq!(result.err().unwrap(), NamingError::PermissionDenied);
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
            NamingError::RegistryNotFoundError {
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
        assert_eq!(result.err().unwrap(), NamingError::PermissionDenied);
    }
}

mod reclaim {
    use super::*;
    use common::named_canister_ids::CanisterNames;

    #[rstest]
    fn test_reclaim_success(
        _init_test: (),
        mut service: RegistriesService,
        mock_user1: Principal,
        resolver: Principal,
    ) {
        let caller = get_named_get_canister_id(CanisterNames::Registrar);

        // act
        service
            .reclaim_name("nice.ic", &caller, &mock_user1, &resolver)
            .unwrap();

        // assert
        STATE.with(|s| {
            let store = s.registry_store.borrow();
            let registries = store.get_registries();
            let registry = registries.get("nice.ic").unwrap();
            assert_eq!(registry.get_name(), "nice.ic");
            assert_eq!(registry.get_resolver(), resolver);
            assert_eq!(registry.get_owner(), &mock_user1);
            assert_eq!(registry.get_ttl(), DEFAULT_TTL);
        })
    }

    #[rstest]
    fn test_reclaim_success_name_owned_by_other_user(
        _init_test: (),
        mut service: RegistriesService,
        mock_user1: Principal,
        mock_user2: Principal,
        resolver: Principal,
    ) {
        let caller = get_named_get_canister_id(CanisterNames::Registrar);
        STATE.with(|s| {
            let mut store = s.registry_store.borrow_mut();
            let registries = store.get_registries_mut();
            registries.insert(
                "nice.ic".to_string(),
                Registry::new(
                    "nice.ic".to_string(),
                    resolver.clone(),
                    0,
                    mock_user2.clone(),
                ),
            );
        });
        // act
        service
            .reclaim_name("nice.ic", &caller, &mock_user1, &resolver)
            .unwrap();

        // assert
        STATE.with(|s| {
            let store = s.registry_store.borrow();
            let registries = store.get_registries();
            let registry = registries.get("nice.ic").unwrap();
            assert_eq!(registry.get_name(), "nice.ic");
            assert_eq!(registry.get_resolver(), resolver);
            assert_eq!(registry.get_owner(), &mock_user1);
            assert_eq!(registry.get_ttl(), DEFAULT_TTL);
        })
    }

    #[rstest]
    fn test_reclaim_failed_caller_error(
        _init_test: (),
        mut service: RegistriesService,
        mock_user1: Principal,
        resolver: Principal,
    ) {
        let _caller = mock_user1;
        // act
        let result = service.reclaim_name("nice.ic", &mock_user1, &mock_user1, &resolver);

        assert_eq!(result.err().unwrap(), NamingError::Unauthorized);
    }
}

mod reset_name {
    use std::borrow::Borrow;

    use common::errors::ErrorInfo;

    use super::*;

    #[rstest]
    async fn test_reset_name_success(
        mut service: RegistriesService,
        mut mock_resolver_api: MockResolverApi,
        mock_user1: Principal,
    ) {
        let names = vec![
            "ssub1.sub1.nice.ic",
            "sub1.nice.ic",
            "sub2.nice.ic",
            "nice.ic",
            "wownice.ic",
            "ic",
        ];
        let resolver = get_named_get_canister_id(CanisterNames::Resolver);

        STATE.with(|s| {
            let mut store = s.registry_store.borrow_mut();
            for name in names.iter() {
                let registry =
                    Registry::new(name.to_string(), mock_user1.clone(), DEFAULT_TTL, resolver);
                store.add_registry(registry);
            }
        });

        mock_resolver_api
            .expect_remove_resolvers()
            .returning(|mut names| {
                names.sort();
                let mut expected = vec![
                    "ssub1.sub1.nice.ic",
                    "sub1.nice.ic",
                    "sub2.nice.ic",
                    "nice.ic",
                ];
                expected.sort();
                assert_eq!(names, expected);
                Ok(true)
            });

        service.resolver_api = Arc::new(mock_resolver_api);

        // act
        let caller = get_named_get_canister_id(CanisterNames::Registrar);
        let resolver = get_named_get_canister_id(CanisterNames::Resolver);
        let result = service.reset_name("nice.ic", &caller, resolver).await;

        // assert
        assert!(result.is_ok());
        STATE.with(|s| {
            let store = s.registry_store.borrow();
            let registries = store.get_registries();
            assert_eq!(registries.len(), 3);
            registries.get("ic").unwrap();
            registries.get("wownice.ic").unwrap();
            let registry = registries.get("nice.ic").unwrap();
            assert_eq!(registry.get_name(), "nice.ic");
            assert_eq!(registry.get_resolver(), resolver);
            assert_eq!(registry.get_owner(), &mock_user1);
            assert_eq!(registry.get_ttl(), DEFAULT_TTL);
        })
    }

    #[rstest]
    async fn test_reset_name_failed_api_error(
        mut service: RegistriesService,
        mut mock_resolver_api: MockResolverApi,
        mock_user1: Principal,
    ) {
        let names = vec![
            "ssub1.sub1.nice.ic",
            "sub1.nice.ic",
            "sub2.nice.ic",
            "nice.ic",
            "wownice.ic",
            "icp",
        ];
        let resolver = get_named_get_canister_id(CanisterNames::Resolver);

        STATE.with(|s| {
            let mut store = s.registry_store.borrow_mut();
            for name in names.iter() {
                let registry =
                    Registry::new(name.to_string(), mock_user1.clone(), DEFAULT_TTL, resolver);
                store.add_registry(registry);
            }
        });

        mock_resolver_api
            .expect_remove_resolvers()
            .returning(|mut names| {
                names.sort();
                let mut expected = vec![
                    "ssub1.sub1.nice.ic",
                    "sub1.nice.ic",
                    "sub2.nice.ic",
                    "nice.ic",
                ];
                expected.sort();
                assert_eq!(names, expected);
                Err(ErrorInfo::from(NamingError::PermissionDenied))
            });

        service.resolver_api = Arc::new(mock_resolver_api);

        // act
        let caller = get_named_get_canister_id(CanisterNames::Registrar);
        let resolver = get_named_get_canister_id(CanisterNames::Resolver);
        let result = service.reset_name("nice.ic", &caller, resolver).await;

        // assert
        assert!(result.is_err());
        STATE.with(|s| {
            let store = s.registry_store.borrow();
            let registries = store.get_registries();
            assert_eq!(registries.len(), names.len());
        })
    }
}

// mod load_state {
//     use std::fs::File;
//     use std::io::Write;
//     use common::dto::decode_zlib;
//     use common::state::StableState;
//     use crate::state::State;
//     use super::*;
//
//     #[rstest]
//     fn get_registration_owners(
//         service: RegistriesService,
//     ) {
//         let zlib = include_bytes!("../../../../local_state_data/registry/latest.zlib");
//         let candi = decode_zlib(zlib);
//         let state = State::decode(candi).unwrap();
//         let store = state.registry_store.borrow();
//         // out to file registrar.csv
//         let mut wtr = csv::Writer::from_writer(vec![]);
//         for (name, registration) in store.get_registries() {
//             wtr.serialize((name, registration.get_owner().to_string())).unwrap();
//         }
//         wtr.flush().unwrap();
//         let csv = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
//         File::create("registry.csv").unwrap().write_all(csv.as_bytes()).unwrap();
//     }
// }

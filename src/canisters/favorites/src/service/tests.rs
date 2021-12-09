use ic_kit::MockContext;
use rstest::*;

use common::errors::ICNSError;
use test_common::ic_api::init_test;
use test_common::user::*;

use super::*;

#[fixture]
fn service() -> ManagerService {
    init_test();
    let context = MockContext::new();
    context.inject();
    ManagerService::new()
}

mod get_favorites {
    use super::*;

    #[rstest]
    fn test_get_favorites_anonymous_user(service: ManagerService, anonymous_user: Principal) {
        let favorites = service.get_favorites(&anonymous_user);
        assert_eq!(favorites.is_err(), true);
        assert_eq!(favorites.unwrap_err(), ICNSError::Unauthorized);
    }

    #[rstest]
    fn get_favorites_empty(service: ManagerService, mock_user1: Principal) {
        let favorites = service.get_favorites(&mock_user1).unwrap();
        assert_eq!(favorites.len(), 0);
    }

    #[rstest]
    fn get_favorites_one(service: ManagerService, mock_user1: Principal) {
        let name = "nice.icp";
        service.add_favorite(&mock_user1, name).unwrap();
        let favorites = service.get_favorites(&mock_user1).unwrap();
        assert_eq!(favorites.len(), 1);
    }
}

mod add_favorite {
    use log::debug;

    use super::*;

    #[rstest]
    fn test_add_favorite_anonymous_user(service: ManagerService, anonymous_user: Principal) {
        let name = "nice.icp";
        let result = service.add_favorite(&anonymous_user, name);
        assert_eq!(result.is_err(), true);
        assert_eq!(result.unwrap_err(), ICNSError::Unauthorized);
    }

    #[rstest]
    #[case("nice.icp", true)]
    #[case(" nice.icp", true)]
    #[case(" nice.icp ", true)]
    #[case(" nice..icp ", false)]
    #[case(".icp ", false)]
    #[case("icp.", false)]
    #[case("icp. ", false)]
    #[case("icp-. ", false)]
    #[case("icp-.nice", true)]
    fn add_favorite_empty(
        service: ManagerService,
        mock_user1: Principal,
        #[case] name: &str,
        #[case] is_ok: bool,
    ) {
        let result = service.add_favorite(&mock_user1, name);
        assert_eq!(result.is_ok(), is_ok);

        if is_ok {
            let favorites = service.get_favorites(&mock_user1).unwrap();
            assert_eq!(favorites.len(), 1);
        }
    }

    #[rstest]
    fn add_favorite_duplicate(service: ManagerService, mock_user1: Principal) {
        let name = "nice.icp";
        service.add_favorite(&mock_user1, name).unwrap();
        let result = service.add_favorite(&mock_user1, name);
        assert_eq!(result.is_ok(), true);
        assert_eq!(service.get_favorites(&mock_user1).unwrap().len(), 1);
    }

    #[rstest]
    fn add_favorite_too_long(service: ManagerService, mock_user1: Principal) {
        let name = "nice.icp";
        let mut long_name = String::from(name);
        for _ in 0..267 {
            long_name.push_str(".icp");
        }
        let result = service.add_favorite(&mock_user1, &long_name);
        assert_eq!(result.is_ok(), false);
        assert_eq!(service.get_favorites(&mock_user1).unwrap().len(), 0);
    }

    #[rstest]
    fn add_favorite_too_many_times(service: ManagerService, mock_user1: Principal) {
        for i in 0..MAX_COUNT_USER_FAVORITES {
            let name = format!("nice{}.icp", i);
            assert_eq!(service.add_favorite(&mock_user1, &name).unwrap(), true);
        }
        let name = "nice.icp";
        let result = service.add_favorite(&mock_user1, name);
        debug!("{:?}", result);
        assert_eq!(result.is_ok(), false);
    }
}

mod remove_favorite {
    use super::*;

    #[rstest]
    fn test_remove_favorite_anonymous_user(service: ManagerService, anonymous_user: Principal) {
        let name = "nice.icp";
        let result = service.remove_favorite(&anonymous_user, name);
        assert_eq!(result.is_err(), true);
        assert_eq!(result.unwrap_err(), ICNSError::Unauthorized);
    }

    #[rstest]
    fn remove_favorite_empty(service: ManagerService, mock_user1: Principal) {
        let result = service.remove_favorite(&mock_user1, "nice.icp");
        assert_eq!(result.is_ok(), true);
    }

    #[rstest]
    fn remove_favorite_one(service: ManagerService, mock_user1: Principal) {
        let name = "nice.icp";
        service.add_favorite(&mock_user1, name).unwrap();
        let result = service.remove_favorite(&mock_user1, name);
        assert_eq!(result.is_ok(), true);

        let favorites = service.get_favorites(&mock_user1).unwrap();
        assert_eq!(favorites.len(), 0);
    }
}

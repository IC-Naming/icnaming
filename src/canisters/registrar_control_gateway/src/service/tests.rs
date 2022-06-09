use rstest::*;

use test_common::canister_api::*;
use test_common::ic_api::init_test;
use test_common::user::*;

use super::*;

#[fixture]
fn service(_init_test: (), mut mock_registrar_api: MockRegistrarApi) -> GatewayService {
    let mut service = GatewayService::default();
    mock_registrar_api
        .expect_register_from_gateway()
        .returning(|_name, _owner| Ok(true));
    service.registrar_api = Arc::new(mock_registrar_api);
    service
}

mod assign_name {
    use common::errors::{ErrorInfo, NamingError};
    use common::permissions::get_admin;

    use super::*;

    #[rstest]
    async fn test_assign_name_success(
        service: GatewayService,
        mock_now: u64,
        mock_user1: Principal,
    ) {
        let caller = get_admin();

        // act
        let result = service
            .assign_name(&caller, mock_now, "icnaming.ic".to_string(), mock_user1)
            .await;

        // assert
        assert!(result.is_ok());
        let result = result.unwrap();
        match result {
            AssignNameResult::Ok => {
                assert!(true);
            }
            _ => {
                assert!(false);
            }
        }
    }

    #[rstest]
    async fn test_assign_name_fail_when_registrar_err(
        mut service: GatewayService,
        mock_now: u64,
        mock_user1: Principal,
        mut mock_registrar_api: MockRegistrarApi,
    ) {
        mock_registrar_api
            .expect_register_from_gateway()
            .returning(|_name, _owner| {
                Err(ErrorInfo::from(NamingError::InvalidName {
                    reason: "invalid name".to_string(),
                }))
            });
        service.registrar_api = Arc::new(mock_registrar_api);

        let caller = get_admin();

        // act
        let result = service
            .assign_name(&caller, mock_now, "icnaming.ic".to_string(), mock_user1)
            .await;

        // assert
        assert!(result.is_ok());
        let result = result.unwrap();
        match result {
            AssignNameResult::FailFromRegistrar => {
                assert!(true);
            }
            _ => {
                assert!(false);
            }
        }
    }

    #[rstest]
    async fn test_assign_name_fail_when_duplicated(
        service: GatewayService,
        mock_now: u64,
        mock_user1: Principal,
    ) {
        let caller = get_admin();

        // act
        let _result = service
            .assign_name(&caller, mock_now, "icnaming.ic".to_string(), mock_user1)
            .await;
        let result = service
            .assign_name(&caller, mock_now, "icnaming.ic".to_string(), mock_user1)
            .await;

        // assert
        assert!(result.is_ok());
        let result = result.unwrap();
        match result {
            AssignNameResult::AlreadyAssigned => {
                assert!(true);
            }
            _ => {
                assert!(false);
            }
        }
    }
}

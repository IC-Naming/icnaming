use super::*;
use common::canister_api::*;
use rstest::*;
use test_common::canister_api::*;
use test_common::ic_api::init_test;
use test_common::user::*;

#[fixture]
fn service(_init_test: (), mut mock_registrar_api: MockRegistrarApi) -> GatewayService {
    let mut service = GatewayService::new();
    mock_registrar_api
        .expect_register_from_gateway()
        .returning(|name, owner| Ok(true));
    service.registrar_api = Arc::new(mock_registrar_api);
    service
}

mod assign_name {
    use super::*;
    use common::errors::{ErrorInfo, ICNSError};
    use common::permissions::get_admin;

    #[rstest]
    async fn test_assign_name_success(
        service: GatewayService,
        mock_now: u64,
        mock_user1: Principal,
    ) {
        let caller = get_admin();

        // act
        let result = service
            .assign_name(&caller, mock_now, "icnaming.icp".to_string(), mock_user1)
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
            .returning(|name, owner| {
                Err(ErrorInfo::from(ICNSError::InvalidName {
                    reason: "invalid name".to_string(),
                }))
            });
        service.registrar_api = Arc::new(mock_registrar_api);

        let caller = get_admin();

        // act
        let result = service
            .assign_name(&caller, mock_now, "icnaming.icp".to_string(), mock_user1)
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
        mut service: GatewayService,
        mock_now: u64,
        mock_user1: Principal,
    ) {
        let caller = get_admin();

        // act
        let result = service
            .assign_name(&caller, mock_now, "icnaming.icp".to_string(), mock_user1)
            .await;
        let result = service
            .assign_name(&caller, mock_now, "icnaming.icp".to_string(), mock_user1)
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

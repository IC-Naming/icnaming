use super::*;
use crate::named_principals::lines_hashset;
use crate::test_common::test::init_test_logger;
use candid::Principal;
use log::info;
use rstest::*;

#[rstest]
fn test_accept_multiline_env() {
    init_test_logger();

    let env_value = NAMING_PRINCIPAL_NAME_STATE_EXPORTER;
    info!("env_value: {}", env_value);
    let set = lines_hashset(env_value);
    assert_eq!(set.len(), 2);
    assert!(set.contains(
        &Principal::from_text("5nzoz-sqvpu-74d7b-5jc7s-lbngl-n72mg-o3vms-imvav-uc4iy-7dxys-nae")
            .unwrap()
    ));
    assert!(set.contains(
        &Principal::from_text("ocmdv-s2awt-ypd2c-uw2e6-v4g4v-qr6gn-wfsim-jo7h4-f3wgu-gokqj-gqe")
            .unwrap()
    ));
}

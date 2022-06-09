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
        &Principal::from_text("ospgt-yzzk6-fvirh-5wvnz-qgpgk-szcnc-6ejpy-h2uqa-q52vv-ugazm-cae")
            .unwrap()
    ));
    assert!(set.contains(
        &Principal::from_text("tjsji-y6o2d-27txe-kwuur-r4nz7-o3ojq-6o22e-t2ss5-3uzqz-lmcuc-tae")
            .unwrap()
    ));
}

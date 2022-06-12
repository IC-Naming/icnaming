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
        &Principal::from_text("2eis6-ev3kx-wr3pi-otbsb-kzzrp-z3oyb-poe6w-bdbtz-gtigi-6ipki-3qe")
            .unwrap()
    ));
    assert!(set.contains(
        &Principal::from_text("ifrhz-krkg5-f3t3h-nxfaf-ttqs5-4zwit-3rady-6nz4z-gh5tu-lenm6-xqe")
            .unwrap()
    ));
}

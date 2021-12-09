use std::str::FromStr;

use ic_cdk::export::Principal;
use rstest::*;

#[fixture]
pub fn anonymous_user() -> Principal {
    Principal::anonymous()
}

#[fixture]
pub fn mock_user1() -> Principal {
    Principal::from_str("zo36k-iqaaa-aaaaj-qahdq-cai").unwrap()
}

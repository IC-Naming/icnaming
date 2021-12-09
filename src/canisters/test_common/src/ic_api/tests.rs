use candid::Principal;
use rstest::*;

use common::ic_api::ic_caller;

use crate::ic_api::init_test;
use crate::ic_api::test_wrapper::set_caller;

#[rstest]
fn test_current_api_caller(_init_test: ()) {
    let caller = ic_caller();
    assert_eq!(caller, Principal::anonymous());
    let current_caller = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
    set_caller(current_caller);
    assert_eq!(ic_caller(), current_caller);
}

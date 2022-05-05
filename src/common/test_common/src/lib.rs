use common::constants::NAMING_TOP_LABEL;

pub mod canister_api;
pub mod ic_api;
pub mod user;

pub fn create_test_name(prefix: &str) -> String {
    format!("{}.{}", prefix, NAMING_TOP_LABEL)
}

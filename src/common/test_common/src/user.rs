use common::canister_api::{AccountIdentifier, Subaccount};
use ic_cdk::export::Principal;
use rstest::*;

#[fixture]
pub fn anonymous_user() -> Principal {
    Principal::anonymous()
}

pub fn mock_user(index: u32) -> Principal {
    let mut principal_bytes = vec![0u8; 29];
    // The first four bytes are the index.
    principal_bytes[0..4].copy_from_slice(&index.to_be_bytes());
    Principal::from_slice(&principal_bytes)
}

pub fn mock_account_id(index: u32, subaccount: u32) -> Vec<u8> {
    // create a subaccount from subaccount
    let mut account_id_bytes = vec![0u8; 32];
    // The first four bytes are the subaccount.
    account_id_bytes[0..4].copy_from_slice(&subaccount.to_be_bytes());
    let account_bytes: [u8; 32] = account_id_bytes.as_slice().try_into().unwrap();
    let subaccount = Subaccount(account_bytes);
    AccountIdentifier::new(mock_user(index), Some(subaccount)).to_vec()
}

pub fn mock_canister(index: u32) -> Principal {
    let mut principal_bytes = vec![0u8; 10];
    // The first four bytes are the index.
    principal_bytes[0..4].copy_from_slice(&index.to_be_bytes());
    Principal::from_slice(&principal_bytes)
}

#[fixture]
pub fn mock_user1() -> Principal {
    mock_user(1)
}

#[fixture]
pub fn mock_user2() -> Principal {
    mock_user(2)
}

#[fixture]
pub fn mock_user3() -> Principal {
    mock_user(3)
}

#[fixture]
pub fn mock_canister1() -> Principal {
    mock_canister(1)
}

#[fixture]
pub fn mock_now() -> u64 {
    15_844_844_000_000_000
}

#[fixture]
pub fn mock_tomorrow() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
        + 1_000_000_000
}

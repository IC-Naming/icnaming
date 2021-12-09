use rstest::*;

use crate::constants::{CANISTER_NAME_ENS_ACTIVITY_CLIENT, CANISTER_NAME_REGISTRY};
use crate::test_common::test::init_test;

use super::*;

#[fixture]
pub fn setup() {
    init_test();
}

#[rstest]
fn test_set_owner(_setup: ()) {
    let owner1 = Principal::from_text("2vxsx-fae").unwrap();
    let owner2 = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
    NAMED_PRINCIPALS.with(|named_principals| {
        let mut named_principals = named_principals.borrow_mut();
        named_principals.set_owner(&owner1, &owner1).unwrap();
        assert!(named_principals.is_owner(&owner1));
        named_principals.set_owner(&owner1, &owner2).unwrap();
        assert!(named_principals.is_owner(&owner2));
        assert_eq!(
            named_principals.set_owner(&owner1, &owner2),
            Err(ICNSError::OwnerOnly),
            "Should have failed because owner2 is not a valid owner"
        );
    });
}

#[rstest]
fn only_owner_can_change_collection(_setup: ()) {
    let owner = Principal::from_text("2vxsx-fae").unwrap();
    let not_owner = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
    let canister_principal = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();

    NAMED_PRINCIPALS.with(|named_principals| {
        let mut named_principals = named_principals.borrow_mut();
        named_principals.set_owner(&owner, &owner).unwrap();
        named_principals
            .add_principal(CANISTER_NAME_REGISTRY, &owner, &canister_principal)
            .unwrap();
        assert_eq!(
            named_principals.add_principal(CANISTER_NAME_REGISTRY, &not_owner, &canister_principal),
            Err(ICNSError::OwnerOnly)
        );
    });
}

#[rstest]
fn add_multiple_named(_setup: ()) {
    let owner = Principal::from_text("2vxsx-fae").unwrap();
    let principal1 = Principal::from_text("2vxsx-fae").unwrap();
    let principal2 = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
    let name1 = CANISTER_NAME_REGISTRY;
    let name2 = CANISTER_NAME_ENS_ACTIVITY_CLIENT;
    NAMED_PRINCIPALS.with(|named_principals| {
        let mut named = named_principals.borrow_mut();
        assert_eq!(named.get_principal(name1), None);
        assert_eq!(named.get_principal(name2), None);
        named.add_principal(name1, &owner, &principal1).unwrap();
        named.add_principal(name2, &owner, &principal2).unwrap();
        assert_eq!(named.get_principal(name1), Some(principal1));
        assert_eq!(named.get_principal(name2), Some(principal2));
    });
}

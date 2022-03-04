use std::str::FromStr;

use candid::Principal;
use once_cell::sync::Lazy;

use crate::errors::{ICNSError, ICNSResult};
use crate::named_canister_ids::is_named_canister_id;
use crate::named_principals::{get_named_principals, is_named_principal, PRINCIPAL_NAME_ADMIN};

pub fn must_be_system_owner(caller: &Principal) -> ICNSResult<()> {
    must_not_anonymous(caller)?;
    if !is_admin(caller) {
        return Err(ICNSError::Unauthorized);
    }
    Ok(())
}

pub fn must_be_named_principal(caller: &Principal, name: &str) -> ICNSResult<()> {
    must_not_anonymous(caller)?;
    if !is_named_principal(name, caller) {
        return Err(ICNSError::Unauthorized);
    }
    Ok(())
}

pub fn must_be_named_canister(caller: &Principal, name: &str) -> ICNSResult<()> {
    must_not_anonymous(caller)?;
    if !is_named_canister_id(name, caller) {
        return Err(ICNSError::Unauthorized);
    }
    Ok(())
}

pub fn must_not_anonymous(caller: &Principal) -> ICNSResult<()> {
    if *caller == Principal::anonymous() {
        return Err(ICNSError::Unauthorized);
    }
    Ok(())
}
pub fn is_admin(user: &Principal) -> bool {
    is_named_principal(PRINCIPAL_NAME_ADMIN, user)
}

#[cfg(feature = "dev_canister")]
pub fn get_admin() -> Principal {
    get_named_principals(PRINCIPAL_NAME_ADMIN)
        .into_iter()
        .next()
        .unwrap()
}

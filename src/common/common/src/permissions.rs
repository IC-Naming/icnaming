use std::str::FromStr;

use candid::Principal;
use once_cell::sync::Lazy;

use crate::errors::{ICNSError, ICNSResult};
use crate::named_principals::{is_named_principal, PRINCIPAL_NAME_ADMIN};

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

pub fn must_not_anonymous(caller: &Principal) -> ICNSResult<()> {
    if *caller == Principal::anonymous() {
        return Err(ICNSError::Unauthorized);
    }
    Ok(())
}
pub fn is_admin(user: &Principal) -> bool {
    is_named_principal(PRINCIPAL_NAME_ADMIN, user)
}

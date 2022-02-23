use crate::errors::{ICNSError, ICNSResult};
use candid::Principal;
use once_cell::sync::Lazy;
use std::str::FromStr;

pub fn must_be_system_owner(caller: &Principal) -> ICNSResult<()> {
    must_not_anonymous(caller)?;
    if !is_admin(caller) {
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

static ADMIN_PRINCIPAL: Lazy<Principal> = Lazy::new(|| {
    let (admin_principal_text,) = {
        #[cfg(feature = "dev_canister")]
        {
            (include_str!(
                "../../../configs/dev/principal_registrar_admin.in"
            ),)
        }
        #[cfg(feature = "staging_canister")]
        {
            (include_str!(
                "../../../configs/staging/principal_registrar_admin.in"
            ),)
        }
        #[cfg(feature = "production_canister")]
        {
            (include_str!(
                "../../../configs/production/principal_registrar_admin.in"
            ),)
        }
    };

    Principal::from_str(admin_principal_text).unwrap()
});
pub fn is_admin(user: &Principal) -> bool {
    ADMIN_PRINCIPAL.eq(user)
}

pub fn get_admin_principal() -> Principal {
    ADMIN_PRINCIPAL.clone()
}

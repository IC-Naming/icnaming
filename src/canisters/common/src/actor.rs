use candid::Principal;
use ic_cdk::api;
use ic_cdk_macros::*;

use crate::constants::ALL_CANISTER_NAMES;
use crate::errors::{to_actor_result, ICNSActorResult, ICNSError};
use crate::ic_api::ic_caller;
use crate::state::NAMED_PRINCIPALS;

#[update(name = "set_owner")]
fn set_owner(owner: Principal) -> ICNSActorResult<()> {
    NAMED_PRINCIPALS.with(|named_principals| {
        let mut caller_settings = named_principals.borrow_mut();
        let result = caller_settings.set_owner(&ic_caller(), &owner);
        let result = to_actor_result(result);
        result
    })
}

#[update(name = "set_named")]
fn set_named(name: String, caller: Principal) -> ICNSActorResult<()> {
    if !ALL_CANISTER_NAMES.contains(&name.as_str()) {
        return to_actor_result(Err(ICNSError::InvalidCanisterName));
    }
    NAMED_PRINCIPALS.with(|named_principals| {
        let mut caller_settings = named_principals.borrow_mut();
        let result = caller_settings.add_principal(name.as_str(), &api::caller(), &caller);
        let result = to_actor_result(result);
        result
    })
}

use std::cell::RefCell;
use std::collections::HashMap;

use candid::{CandidType, Deserialize, Principal};
use log::info;

use crate::errors::{ICNSError, ICNSResult};

#[cfg(test)]
mod tests;

thread_local! {
    pub static NAMED_PRINCIPALS: RefCell<NamedPrincipal> = RefCell::new(NamedPrincipal::new());
}

pub struct NamedPrincipal {
    owner: Principal,
    named_principals: HashMap<String, Principal>,
}

#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct NamedPrincipalStable {
    owner: Principal,
    named_principals: HashMap<String, Principal>,
}

impl Default for NamedPrincipal {
    fn default() -> Self {
        NamedPrincipal {
            owner: Principal::anonymous(),
            named_principals: HashMap::new(),
        }
    }
}

impl NamedPrincipal {
    pub fn new() -> Self {
        NamedPrincipal::default()
    }

    pub fn set_owner(&mut self, caller: &Principal, new_owner: &Principal) -> ICNSResult<()> {
        // set owner if is anonymous or this current
        if self.owner == Principal::anonymous() {
            info!("current owner not found, set owner to: {}", new_owner);
            self.owner = new_owner.clone();
            Ok(())
        } else if self.owner == *caller {
            info!("owner from {} to: {}", self.owner, new_owner);
            self.owner = new_owner.clone();
            Ok(())
        } else {
            Err(ICNSError::OwnerOnly)
        }
    }

    fn is_owner(&self, caller: &Principal) -> bool {
        self.owner == *caller
    }

    pub fn add_principal(
        &mut self,
        name: &str,
        caller: &Principal,
        new: &Principal,
    ) -> ICNSResult<()> {
        if &self.owner != caller {
            return Err(ICNSError::OwnerOnly);
        }
        let old_one = self.named_principals.insert(name.to_string(), new.clone());
        if old_one.is_some() {
            info!(
                "Key {} ,Principal updated from {} to {}",
                name,
                old_one.unwrap(),
                new
            );
        } else {
            info!("Key {} ,Principal added {}", name, new);
        }
        Ok(())
    }

    fn get_principal(&self, name: &str) -> Option<Principal> {
        self.named_principals.get(name).cloned()
    }

    fn is_principal(&self, name: &str, caller: &Principal) -> bool {
        let config = self.named_principals.get(name).unwrap();
        if config == caller {
            return true;
        }
        false
    }
}

pub fn set_named_principal_owner(caller: &Principal, new_owner: &Principal) -> ICNSResult<()> {
    NAMED_PRINCIPALS.with(|named_principals| {
        let mut named_principals = named_principals.borrow_mut();
        named_principals.set_owner(caller, new_owner)
    })
}

pub fn add_principal(name: &str, caller: &Principal, new: &Principal) -> ICNSResult<()> {
    NAMED_PRINCIPALS.with(|named_principals| {
        let mut named_principals = named_principals.borrow_mut();
        named_principals.add_principal(name, caller, new)
    })
}

pub fn is_owner(caller: &Principal) -> bool {
    NAMED_PRINCIPALS.with(|named_principals| {
        let named_principals = named_principals.borrow();
        named_principals.is_owner(caller)
    })
}

pub fn is_principal(name: &str, caller: &Principal) -> bool {
    NAMED_PRINCIPALS.with(|named_principals| {
        let named_principals = named_principals.borrow();
        named_principals.is_principal(name, caller)
    })
}

pub fn get_principal(name: &str) -> Option<Principal> {
    NAMED_PRINCIPALS.with(|named_principals| {
        let named_principals = named_principals.borrow();
        named_principals.get_principal(name)
    })
}

pub fn get_stable_named_principal() -> NamedPrincipalStable {
    NAMED_PRINCIPALS.with(|named_principals| {
        let named_principals = named_principals.borrow();
        NamedPrincipalStable {
            owner: named_principals.owner.clone(),
            named_principals: named_principals.named_principals.clone(),
        }
    })
}

pub fn load_stable_named_principal(stable: &NamedPrincipalStable) {
    NAMED_PRINCIPALS.with(|named_principals| {
        let mut named_principals = named_principals.borrow_mut();
        named_principals.owner = stable.owner.clone();
        named_principals.named_principals = stable.named_principals.clone();
    })
}

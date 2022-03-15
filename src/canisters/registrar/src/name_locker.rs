use log::{debug, error};
use std::collections::HashSet;

use common::errors::{ICNSError, ICNSResult};

use crate::state::NAME_LOCKER;

pub struct NameLocker {
    locks: HashSet<String>,
}

impl NameLocker {
    pub fn new() -> Self {
        Self {
            locks: HashSet::new(),
        }
    }

    pub fn lock(&mut self, name: &str) -> bool {
        let new_insert = self.locks.insert(name.to_string());
        if new_insert {
            debug!("Locked name: {}", name);
        } else {
            error!("Name already locked: {}", name);
        }
        new_insert
    }

    pub fn unlock(&mut self, name: &str) -> bool {
        let removed = self.locks.remove(name);
        if removed {
            debug!("Unlocked name: {}", name);
        } else {
            error!("Name not locked: {}", name);
        }
        removed
    }

    pub fn get_count(&self) -> u32 {
        self.locks.len() as u32
    }

    pub fn is_locked(&self, name: &str) -> bool {
        self.locks.contains(name)
    }
}

pub fn try_lock_name(name: &str) -> ICNSResult<()> {
    NAME_LOCKER.with(|locker| {
        let mut locker = locker.borrow_mut();
        if locker.is_locked(name) {
            Err(ICNSError::Conflict)
        } else {
            locker.lock(name);
            Ok(())
        }
    })
}

pub fn unlock_name(name: &str) {
    NAME_LOCKER.with(|locker| {
        let mut locker = locker.borrow_mut();
        locker.unlock(name);
    });
}

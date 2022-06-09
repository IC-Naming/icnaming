use crate::TimeInNs;
use core::cell::RefCell;
use std::collections::HashMap;

#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum LockId {
    TokenServiceRefund,
}

// 60 seconds
const LOCKER_TIMEOUT_NS: TimeInNs = TimeInNs(60_000_000_000);

thread_local! {
    static TIMEOUT_LOCKS: RefCell<TimeoutLocker> = RefCell::new(TimeoutLocker::default());
}

#[derive(Default, Debug)]
pub struct TimeoutLocker {
    lockers: HashMap<LockId, TimeInNs>,
}

impl TimeoutLocker {
    pub fn try_lock(&mut self, lock_id: LockId, now: TimeInNs) -> bool {
        if let Some(lock_time) = self.lockers.get(&lock_id) {
            if now - *lock_time > LOCKER_TIMEOUT_NS {
                self.lockers.insert(lock_id, now);
                return true;
            }
        } else {
            self.lockers.insert(lock_id, now);
            return true;
        }
        false
    }

    pub fn release(&mut self, lock_id: LockId) {
        self.lockers.remove(&lock_id);
    }
}

pub fn try_lock_with_timeout(lock_id: LockId, now: TimeInNs) -> bool {
    TIMEOUT_LOCKS.with(|locker| {
        let mut locker = locker.borrow_mut();
        locker.try_lock(lock_id, now)
    })
}

pub fn release_timeout_locker(lock_id: LockId) {
    TIMEOUT_LOCKS.with(|locker| {
        let mut locker = locker.borrow_mut();
        locker.release(lock_id)
    })
}

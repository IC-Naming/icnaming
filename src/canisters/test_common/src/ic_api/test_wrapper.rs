use std::cell::RefCell;
use std::sync::Arc;

use candid::Principal;
use log::info;

use common::ic_api::{IClock, IRequestContext, IC_CLOCK, IC_REQUEST_CONTEXT};

thread_local! {
    pub static current_caller: RefCell<Principal> = RefCell::new( Principal::anonymous());
    pub static current_time: RefCell<u64> = RefCell::new( 0);
}

pub fn enable_test_ic_api() {
    IC_CLOCK.with(|clock| {
        let mut clock = clock.borrow_mut();
        *clock = Arc::new(TestICApi {});
    });
    IC_REQUEST_CONTEXT.with(|context| {
        let mut context = context.borrow_mut();
        *context = Arc::new(TestICApi {});
    });
}

pub fn set_caller(caller: Principal) {
    info!("Setting caller to {}", caller);
    current_caller.with(|c| *c.borrow_mut() = caller);
}

pub struct TestICApi {}

impl TestICApi {
    pub fn new() -> TestICApi {
        TestICApi {}
    }
    pub fn set_now_ms(&mut self, now: u64) {
        current_time.with(|c| *c.borrow_mut() = now * 1_000_000);
    }
}

impl IClock for TestICApi {
    fn now_ms(&self) -> u64 {
        self.now_ns() / 1_000_000
    }

    fn now_s(&self) -> u64 {
        self.now_ms() / 1_000
    }

    fn now_ns(&self) -> u64 {
        current_time.with(|c| c.borrow().clone())
    }
}

impl IRequestContext for TestICApi {
    fn get_caller(&self) -> Principal {
        current_caller.with(|c| c.borrow().clone())
    }
}

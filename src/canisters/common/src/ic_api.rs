use std::cell::RefCell;
use std::sync::Arc;

use candid::Principal;

use crate::ic_api::wrapper::ICStaticApi;

pub mod wrapper;
thread_local! {
    pub static IC_CLOCK: RefCell<Arc<dyn IClock>> = RefCell::new(Arc::new(ICStaticApi::new()));
    pub static IC_REQUEST_CONTEXT: RefCell<Arc<dyn IRequestContext>> = RefCell::new(Arc::new(ICStaticApi::new()));
}
pub fn ic_caller() -> Principal {
    IC_REQUEST_CONTEXT.with(|rc| rc.borrow().get_caller())
}

pub fn ic_now() -> u64 {
    IC_CLOCK.with(|c| c.borrow().now_ns())
}

pub trait IClock {
    fn now_ms(&self) -> u64;
    fn now_s(&self) -> u64;
    fn now_ns(&self) -> u64;
}

pub trait IRequestContext {
    fn get_caller(&self) -> Principal;
}

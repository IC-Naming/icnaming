use candid::Principal;
use ic_cdk::api;

use super::*;

pub struct ICStaticApi;

impl ICStaticApi {
    pub fn new() -> Self {
        ICStaticApi
    }
}

impl IClock for ICStaticApi {
    fn now_ms(&self) -> u64 {
        self.now_ns() / 1_000_000
    }

    fn now_s(&self) -> u64 {
        self.now_ms() / 1_000
    }

    fn now_ns(&self) -> u64 {
        api::time()
    }
}

impl IRequestContext for ICStaticApi {
    fn get_caller(&self) -> Principal {
        api::caller()
    }
}

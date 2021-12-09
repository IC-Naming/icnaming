use log::LevelFilter;
use rstest::*;

use crate::ic_api::test_wrapper::enable_test_ic_api;

pub mod test_wrapper;
#[cfg(test)]
mod tests;

pub fn init_test_logger() {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Trace)
        .is_test(true)
        .try_init();
}

#[fixture]
pub fn init_test() {
    init_test_logger();
    enable_test_ic_api();
}

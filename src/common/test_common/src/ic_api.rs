use log::LevelFilter;
use rstest::*;

pub fn init_test_logger() {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Trace)
        .is_test(true)
        .try_init();
}

#[fixture]
pub fn init_test() {
    init_test_logger();
}

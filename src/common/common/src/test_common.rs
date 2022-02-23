#[cfg(test)]
pub(crate) mod test {
    use std::sync::Once;

    use log::{info, LevelFilter};

    static INIT: Once = Once::new();

    pub fn init_test_logger() {
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Trace)
            .is_test(true)
            .try_init();
    }

    pub(crate) fn init_test() {
        INIT.call_once(|| {
            init_test_logger();
            info!("init_test");
        });
    }
}

use candid::encode_args;

use common::state::StableState;

#[derive(Default)]
pub struct Settings;

impl StableState for Settings {
    fn encode(&self) -> Vec<u8> {
        encode_args((0,)).unwrap()
    }

    fn decode(_: Vec<u8>) -> Result<Self, String> {
        Ok(Settings {})
    }
}

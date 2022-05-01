use common::state::StableState;

thread_local! {
    pub static STATE : State = State::default();
}

#[derive(Default)]
pub struct State {
    // NOTE: When adding new persistent fields here, ensure that these fields
    // are being persisted in the `replace` method below.
}

impl State {
    pub fn replace(&self, _new_state: State) {
        unreachable!("State should not be replaced");
    }
}

impl StableState for State {
    fn encode(&self) -> Vec<u8> {
        unreachable!("State should not be encoded");
    }

    fn decode(_bytes: Vec<u8>) -> Result<Self, String> {
        unreachable!("State::decode should never be called");
    }
}

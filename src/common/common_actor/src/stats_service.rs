use candid::{CandidType, Deserialize};

#[derive(Default)]
pub struct StatsService {}

impl StatsService {
    pub fn get_stats(&self, _now: u64) -> Stats {
        unreachable!("StatsService::get_stats()")
    }
}

#[derive(CandidType, Deserialize)]
pub struct Stats {}

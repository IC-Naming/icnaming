use crate::state::STATE;
use candid::{CandidType, Deserialize};
use common::metrics_encoder::MetricsEncoder;
use ic_cdk::api;

#[derive(Default)]
pub struct StatsService {}

impl StatsService {
    pub fn get_stats(&self, _now: u64) -> Stats {
        let mut stats = Stats::default();
        stats.cycles_balance = api::canister_balance();
        STATE.with(|s| {
            let user_set = s.user_favorite_store.borrow();
            let user_favorites = user_set.get_favorites();
            stats.user_count = user_favorites.len() as u64;
            stats.favorite_count = user_favorites
                .values()
                .fold(0u64, |acc, favorites| acc + favorites.len() as u64);
        });
        stats
    }
}

pub fn encode_metrics(w: &mut MetricsEncoder<Vec<u8>>, now: u64) -> std::io::Result<()> {
    let service = StatsService::default();
    let stats = service.get_stats(now);
    w.encode_gauge(
        "icnaming_favorites_cycles_balance",
        stats.cycles_balance as f64,
        "Balance in cycles",
    )?;
    w.encode_gauge(
        "icnaming_favorites_user_count",
        stats.user_count as f64,
        "Number of users",
    )?;
    w.encode_gauge(
        "icnaming_favorites_favorite_count",
        stats.favorite_count as f64,
        "Number of favorites",
    )?;

    Ok(())
}

#[derive(CandidType, Deserialize, Default)]
pub struct Stats {
    cycles_balance: u64,
    user_count: u64,
    favorite_count: u64,
}

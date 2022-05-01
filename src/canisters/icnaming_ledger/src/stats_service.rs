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

        stats
    }
}

pub fn encode_metrics(w: &mut MetricsEncoder<Vec<u8>>, now: u64) -> std::io::Result<()> {
    let service = StatsService::default();
    let stats = service.get_stats(now);
    w.encode_gauge(
        "icnaming_ledger_cycles_balance",
        stats.cycles_balance as f64,
        "Balance in cycles",
    )?;

    Ok(())
}

#[derive(CandidType, Deserialize, Default)]
pub struct Stats {
    cycles_balance: u64,
}

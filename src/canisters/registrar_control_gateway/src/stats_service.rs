use crate::state::STATE;
use candid::{CandidType, Deserialize};
use common::metrics_encoder::MetricsEncoder;
use ic_cdk::api;

#[derive(Default)]
pub struct StatsService {}

impl StatsService {
    pub(crate) fn get_stats(&self, _now: u64) -> Stats {
        let mut stats = Stats::default();
        stats.cycles_balance = api::canister_balance();
        STATE.with(|s| {
            {
                let store = s.name_assignment_store.borrow();
                let assignments = store.get_assignments();
                stats.name_assignments_count = assignments.len() as u64;
            }
            {
                let store = s.quota_import_store.borrow();
                let acceptable_file_hashes = store.get_acceptable_file_hashes();
                stats.acceptable_file_hashes_count = acceptable_file_hashes.len() as u64;

                let imported_file_hashes = store.get_imported_file_hashes();
                stats.imported_file_hashes_count = imported_file_hashes.len() as u64;
            }
        });

        stats
    }
}

pub fn encode_metrics(w: &mut MetricsEncoder<Vec<u8>>, now: u64) -> std::io::Result<()> {
    let service = StatsService::default();
    let stats = service.get_stats(now);
    w.encode_gauge(
        "icnaming_registrar_control_gateway_cycles_balance",
        stats.cycles_balance as f64,
        "Cycles balance",
    )?;
    w.encode_gauge(
        "icnaming_registrar_control_gateway_acceptable_file_hashes_count",
        stats.acceptable_file_hashes_count as f64,
        "Acceptable file hashes count",
    )?;
    w.encode_gauge(
        "icnaming_registrar_control_gateway_imported_file_hashes_count",
        stats.imported_file_hashes_count as f64,
        "Imported file hashes count",
    )?;
    w.encode_gauge(
        "icnaming_registrar_control_gateway_name_assignments_count",
        stats.name_assignments_count as f64,
        "Assigned names count",
    )?;

    Ok(())
}

#[derive(CandidType, Deserialize, Default)]
pub struct Stats {
    cycles_balance: u64,
    acceptable_file_hashes_count: u64,
    imported_file_hashes_count: u64,
    name_assignments_count: u64,
}

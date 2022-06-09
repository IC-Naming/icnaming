use std::collections::{HashMap, HashSet};

use candid::{CandidType, Deserialize};
use ic_cdk::api;

use common::metrics_encoder::MetricsEncoder;

use crate::name_order_store::NameOrderStatus;

use crate::state::*;
use crate::user_quota_store::QuotaType;

#[derive(Default)]
pub struct StatsService {}

impl StatsService {
    pub fn get_stats(&self, _now: u64) -> Stats {
        let mut stats = Stats::default();
        stats.cycles_balance = api::canister_balance();
        NAME_LOCKER.with(|name_locker| {
            stats.name_lock_count = name_locker.borrow().get_count() as u64;
        });
        STATE.with(|s| {
            {
                let store = s.registration_store.borrow();
                // count distinct owners of registered names
                let mut owners = HashSet::new();
                for registration in store.get_registrations().values() {
                    owners.insert(registration.get_owner());
                }
                stats.user_count = owners.len() as u64;
            }
            {
                let mut user_quota_count = HashMap::new();
                let quota_types = vec![
                    QuotaType::LenGte(1),
                    QuotaType::LenGte(2),
                    QuotaType::LenGte(3),
                    QuotaType::LenGte(4),
                    QuotaType::LenGte(5),
                    QuotaType::LenGte(6),
                    QuotaType::LenGte(7),
                    QuotaType::LenEq(1),
                    QuotaType::LenEq(2),
                    QuotaType::LenEq(3),
                    QuotaType::LenEq(4),
                    QuotaType::LenEq(5),
                    QuotaType::LenEq(6),
                    QuotaType::LenEq(7),
                ];
                for quota_type in quota_types {
                    user_quota_count.insert(quota_type, 0u64);
                }
                let store = s.user_quota_store.borrow();
                let quotas = store.get_user_quotas();
                for user_quota in quotas.values() {
                    for (t, user_count) in user_quota {
                        let count = user_quota_count.entry(*t).or_insert(0);
                        *count += *user_count as u64;
                    }
                }
                let mut user_quota_count_stats = HashMap::new();
                for (quota_type, count) in user_quota_count {
                    let type_str = quota_type.to_string().replace('(', "").replace(')', "");
                    user_quota_count_stats.entry(type_str).or_insert(count);
                }
                stats.user_quota_count = user_quota_count_stats;
            }
            {
                let store = s.name_order_store.borrow();
                let orders = store.get_all_orders();
                let mut count_by_status = HashMap::new();
                count_by_status.insert(format!("{:?}", NameOrderStatus::New).to_lowercase(), 0);
                count_by_status.insert(format!("{:?}", NameOrderStatus::Done).to_lowercase(), 0);
                count_by_status.insert(
                    format!("{:?}", NameOrderStatus::WaitingToRefund).to_lowercase(),
                    0,
                );
                count_by_status
                    .insert(format!("{:?}", NameOrderStatus::Canceled).to_lowercase(), 0);

                for (_, order) in orders {
                    let status: &NameOrderStatus = order.order_status();
                    let count = count_by_status
                        .entry(format!("{:?}", status).to_lowercase())
                        .or_insert(0);
                    *count += 1;
                }

                stats.user_name_order_count_by_status = count_by_status;
            }
            {
                let store = s.registration_store.borrow();
                let count = store.get_registrations().len();

                stats.registration_count = count as u64;
            }
            {
                let store = s.payment_store.borrow();
                stats.payment_version = store.get_payment_version_synced_up_to().unwrap_or(0);
            }
        });
        MERTRICS_COUNTER.with(|c| {
            let counter = c.borrow();
            stats.last_xdr_permyriad_per_icp = counter.last_xdr_permyriad_per_icp;
            stats.last_timestamp_seconds_xdr_permyriad_per_icp =
                counter.last_timestamp_seconds_xdr_permyriad_per_icp;
            stats.name_order_placed_count = counter.name_order_placed_count;
            stats.name_order_paid_count = counter.name_order_paid_count;
            stats.name_order_cancelled_count = counter.name_order_cancelled_count;
            stats.new_registered_name_count = counter.new_registered_name_count;
        });

        stats
    }
}

pub fn encode_metrics(w: &mut MetricsEncoder<Vec<u8>>, now: u64) -> std::io::Result<()> {
    let service = StatsService::default();
    let stats = service.get_stats(now);
    for (status, count) in stats.user_quota_order_count.iter() {
        w.encode_gauge(
            format!("icnaming_registrar_quota_order_status_{}", status).as_str(),
            *count as f64,
            format!("Number of quota orders with status {}", status).as_str(),
        )?;
    }
    for (status, count) in stats.user_name_order_count_by_status.iter() {
        w.encode_gauge(
            format!("icnaming_registrar_name_order_status_{}", status).as_str(),
            *count as f64,
            format!("Number of name orders with status {}", status).as_str(),
        )?;
    }
    for (t, count) in stats.user_quota_count.iter() {
        if count > &0u64 {
            w.encode_gauge(
                format!("icnaming_registrar_quota_type_{}", t).as_str(),
                *count as f64,
                format!("Number of quotas with type {}", t).as_str(),
            )?;
        }
    }
    w.encode_gauge(
        "icnaming_registrar_user_count",
        stats.user_count as f64,
        "Number of users",
    )?;
    w.encode_gauge(
        "icnaming_registrar_registrations_count",
        stats.registration_count as f64,
        "Number of registrations",
    )?;
    w.encode_gauge(
        "icnaming_registrar_last_xdr_permyriad_per_icp",
        stats.last_xdr_permyriad_per_icp as f64,
        "Last XDR permyriad per ICP",
    )?;
    w.encode_gauge(
        "icnaming_registrar_last_timestamp_seconds_xdr_permyriad_per_icp",
        stats.last_timestamp_seconds_xdr_permyriad_per_icp as f64,
        "Last timestamp seconds XDR permyriad per ICP",
    )?;
    w.encode_counter(
        "icnaming_registrar_name_order_placed_count",
        stats.name_order_placed_count as f64,
        "Number of name orders placed",
    )?;
    w.encode_counter(
        "icnaming_registrar_name_order_paid_count",
        stats.name_order_paid_count as f64,
        "Number of name orders paid",
    )?;
    w.encode_counter(
        "icnaming_registrar_name_order_cancelled_count",
        stats.name_order_cancelled_count as f64,
        "Number of name orders cancelled",
    )?;
    w.encode_counter(
        "icnaming_registrar_new_registered_name_count",
        stats.new_registered_name_count as f64,
        "Number of new registered names",
    )?;
    w.encode_gauge(
        "icnaming_registrar_payment_version",
        stats.payment_version as f64,
        "Payment version synced",
    )?;
    w.encode_gauge(
        "icnaming_registrar_cycles_balance",
        stats.cycles_balance as f64,
        "Cycles balance",
    )?;

    Ok(())
}

#[derive(CandidType, Deserialize, Default)]
pub struct Stats {
    cycles_balance: u64,
    user_count: u64,
    // obsolete: user_quota_order_count
    user_quota_order_count: HashMap<String, u64>,
    user_quota_count: HashMap<String, u64>,
    user_name_order_count_by_status: HashMap<String, u64>,
    registration_count: u64,
    last_xdr_permyriad_per_icp: u64,
    last_timestamp_seconds_xdr_permyriad_per_icp: u64,
    name_order_placed_count: u64,
    name_order_paid_count: u64,
    name_order_cancelled_count: u64,
    new_registered_name_count: u64,
    payment_version: u64,
    name_lock_count: u64,
}

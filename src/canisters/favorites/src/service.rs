use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api;

use common::constants::{MAX_COUNT_USER_FAVORITES, MAX_LENGTH_USER_FAVORITES};
use common::errors::ICNSError::{InvalidName, TooManyFavorites, ValueMaxLengthError};
use common::errors::ICNSResult;
use common::metrics_encoder::MetricsEncoder;
use common::naming::{normalize_name, parse_name};
use common::permissions::must_not_anonymous;

use crate::state::STATE;

pub(crate) struct ManagerService {}

impl ManagerService {
    pub fn new() -> Self {
        ManagerService {}
    }
}

impl ManagerService {
    pub fn get_favorites(&self, user: &Principal) -> ICNSResult<Vec<String>> {
        must_not_anonymous(user)?;
        STATE.with(|s| {
            let user_set = s.user_favorite_store.borrow();
            let favorites = user_set.get_user_favorites(user);
            Ok(favorites
                .map(|favorites| favorites.clone())
                .unwrap_or(vec![]))
        })
    }

    pub fn add_favorite(&self, now: u64, user: &Principal, favorite: &str) -> ICNSResult<bool> {
        must_not_anonymous(user)?;
        let favorite = normalize_name(favorite);
        if favorite.len() == 0 {
            return Err(InvalidName {
                reason: "empty name".to_string(),
            });
        }
        let max_length = MAX_LENGTH_USER_FAVORITES;
        if favorite.len() > max_length {
            return Err(ValueMaxLengthError { max: max_length });
        }
        let result = parse_name(&favorite);
        if result.is_err() {
            return Err(InvalidName {
                reason: result.err().unwrap(),
            });
        }

        STATE.with(|s| {
            let mut user_set = s.user_favorite_store.borrow_mut();
            if let Some(items) = user_set.get_user_favorites(user) {
                let max_count = MAX_COUNT_USER_FAVORITES;
                if items.len() >= max_count {
                    return Err(TooManyFavorites { max: max_count });
                }

                // skip duplicates
                if items.contains(&favorite) {
                    return Ok(true);
                }
            }
            user_set.add_favorite(user, now, favorite.as_str());
            Ok(true)
        })
    }

    pub fn remove_favorite(&self, now: u64, user: &Principal, favorite: &str) -> ICNSResult<bool> {
        must_not_anonymous(user)?;
        let favorite = normalize_name(favorite);
        let result = parse_name(&favorite);
        if result.is_err() {
            return Err(InvalidName {
                reason: result.err().unwrap(),
            });
        }

        STATE.with(|s| {
            let mut user_set = s.user_favorite_store.borrow_mut();
            user_set.remove_favorite(user, now, favorite.as_str());
            Ok(true)
        })
    }

    pub fn get_stats(&self) -> Stats {
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

pub fn encode_metrics(w: &mut MetricsEncoder<Vec<u8>>) -> std::io::Result<()> {
    let service = ManagerService::new();
    let stats = service.get_stats();
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

#[cfg(test)]
mod tests;

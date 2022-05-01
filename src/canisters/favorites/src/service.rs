use candid::Principal;

use common::constants::{MAX_COUNT_USER_FAVORITES, MAX_LENGTH_USER_FAVORITES};
use common::errors::NamingError::{InvalidName, TooManyFavorites, ValueMaxLengthError};
use common::errors::ServiceResult;

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
    pub fn get_favorites(&self, user: &Principal) -> ServiceResult<Vec<String>> {
        must_not_anonymous(user)?;
        STATE.with(|s| {
            let user_set = s.user_favorite_store.borrow();
            let favorites = user_set.get_user_favorites(user);
            Ok(favorites
                .map(|favorites| favorites.clone())
                .unwrap_or(vec![]))
        })
    }

    pub fn add_favorite(&self, now: u64, user: &Principal, favorite: &str) -> ServiceResult<bool> {
        must_not_anonymous(user)?;
        let favorite = normalize_name(favorite).0;
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

    pub fn remove_favorite(
        &self,
        now: u64,
        user: &Principal,
        favorite: &str,
    ) -> ServiceResult<bool> {
        must_not_anonymous(user)?;
        let favorite = normalize_name(favorite).0;
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
}

#[cfg(test)]
mod tests;

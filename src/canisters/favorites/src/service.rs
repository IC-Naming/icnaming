use ic_kit::Principal;

use common::constants::{MAX_COUNT_USER_FAVORITES, MAX_LENGTH_USER_FAVORITES};
use common::errors::ICNSError::{
    InvalidName, TooManyFavorites, Unauthorized, ValueMaxLengthError,
};
use common::errors::ICNSResult;
use common::naming::{normalize_name, parse_name};

use crate::state::USER_SET;

#[cfg(test)]
mod tests;

pub(crate) struct ManagerService {}

impl ManagerService {
    pub fn new() -> Self {
        ManagerService {}
    }
}

fn is_valid_user(user: &Principal) -> ICNSResult<()> {
    if user.to_owned() != Principal::anonymous() {
        Ok(())
    } else {
        Err(Unauthorized)
    }
}

impl ManagerService {
    pub fn get_favorites(&self, user: &Principal) -> ICNSResult<Vec<String>> {
        is_valid_user(user)?;
        USER_SET.with(|user_set| {
            let user_set = user_set.borrow();
            let favorites = user_set.get_favorites(user);
            Ok(favorites
                .map(|favorites| favorites.clone())
                .unwrap_or(vec![]))
        })
    }

    pub fn add_favorite(&self, user: &Principal, favorite: &str) -> ICNSResult<bool> {
        is_valid_user(user)?;
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

        USER_SET.with(|user_set| {
            let mut user_set = user_set.borrow_mut();
            if let Some(items) = user_set.get_favorites(user) {
                let max_count = MAX_COUNT_USER_FAVORITES;
                if items.len() >= max_count {
                    return Err(TooManyFavorites { max: max_count });
                }

                // skip duplicates
                if items.contains(&favorite) {
                    return Ok(true);
                }
            }
            user_set.add_favorite(user, favorite.as_str());
            Ok(true)
        })
    }

    pub fn remove_favorite(&self, user: &Principal, favorite: &str) -> ICNSResult<bool> {
        is_valid_user(user)?;
        let favorite = normalize_name(favorite);
        let result = parse_name(&favorite);
        if result.is_err() {
            return Err(InvalidName {
                reason: result.err().unwrap(),
            });
        }

        USER_SET.with(|user_set| {
            let mut user_set = user_set.borrow_mut();
            user_set.remove_favorite(user, favorite.as_str());
            Ok(true)
        })
    }
}

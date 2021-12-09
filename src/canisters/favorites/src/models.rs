use std::collections::HashMap;

use ic_cdk::export::Principal;
use ic_kit::ic;

use crate::state::{UserFavoriteSetStable, UserFavoriteStable};

pub(crate) struct UserFavoriteSet {
    user_favorites: HashMap<Principal, UserFavorite>,
}

impl From<&UserFavoriteSetStable> for UserFavoriteSet {
    fn from(user_favorites: &UserFavoriteSetStable) -> Self {
        let mut result: HashMap<Principal, UserFavorite> = HashMap::new();
        for (principal, user_favorite) in user_favorites.user_favorites.iter() {
            result.insert(principal.clone(), user_favorite.into());
        }
        Self {
            user_favorites: result,
        }
    }
}

impl From<&UserFavoriteSet> for UserFavoriteSetStable {
    fn from(user_favorites: &UserFavoriteSet) -> Self {
        let mut result: HashMap<Principal, UserFavoriteStable> = HashMap::new();
        for (principal, user_favorite) in user_favorites.user_favorites.iter() {
            result.insert(principal.clone(), user_favorite.into());
        }
        Self {
            user_favorites: result,
        }
    }
}

impl UserFavoriteSet {
    pub fn new() -> Self {
        UserFavoriteSet {
            user_favorites: HashMap::new(),
        }
    }

    pub fn get_favorites(&self, user: &Principal) -> Option<&Vec<String>> {
        if let Some(user_favorites) = self.user_favorites.get(user) {
            Some(&user_favorites.favorites)
        } else {
            None
        }
    }

    pub fn add_favorite(&mut self, user: &Principal, favorite: &str) {
        let user_favorite = self
            .user_favorites
            .entry(user.to_owned())
            .or_insert_with(|| UserFavorite::new());
        user_favorite.add_favorite(favorite);
    }

    pub fn remove_favorite(&mut self, user: &Principal, favorite: &str) {
        let items = self.user_favorites.get_mut(&user.to_owned());
        if let Some(items) = items {
            items.remove_favorite(favorite);
            if items.is_empty() {
                self.user_favorites.remove(&user);
            }
        }
    }
}

pub(crate) struct UserFavorite {
    favorites: Vec<String>,
    last_update_time: u64,
}

impl From<&UserFavoriteStable> for UserFavorite {
    fn from(user_favorite_stable: &UserFavoriteStable) -> Self {
        UserFavorite {
            favorites: user_favorite_stable.favorites.clone(),
            last_update_time: user_favorite_stable.last_update_time,
        }
    }
}

impl From<&UserFavorite> for UserFavoriteStable {
    fn from(user_favorite: &UserFavorite) -> Self {
        UserFavoriteStable {
            favorites: user_favorite.favorites.clone(),
            last_update_time: user_favorite.last_update_time,
        }
    }
}

impl UserFavorite {
    pub fn new() -> Self {
        UserFavorite {
            favorites: Vec::new(),
            last_update_time: 0,
        }
    }

    pub fn add_favorite(&mut self, favorite: &str) {
        self.favorites.insert(0, favorite.to_string());
        self.last_update_time = ic::time();
    }

    pub fn get_favorites(&self) -> &Vec<String> {
        &self.favorites
    }

    pub fn get_last_update_time(&self) -> u64 {
        self.last_update_time
    }

    pub fn remove_favorite(&mut self, favorite: &str) {
        self.favorites.retain(|f| f != favorite);
        self.last_update_time = ic::time();
    }

    pub fn is_empty(&self) -> bool {
        self.favorites.is_empty()
    }
    pub fn len(&self) -> usize {
        self.favorites.len()
    }
}

use std::collections::HashMap;

use candid::{CandidType, decode_args, Deserialize, encode_args};
use ic_cdk::export::Principal;
use log::debug;

use common::state::StableState;

#[derive(CandidType, Deserialize)]
pub(crate) struct UserFavorite {
    favorites: Vec<String>,
    last_update_time: u64,
}

impl UserFavorite {
    pub fn new() -> Self {
        UserFavorite {
            favorites: Vec::new(),
            last_update_time: 0,
        }
    }

    pub fn add_favorite(&mut self, favorite: &str, now: u64) {
        self.favorites.insert(0, favorite.to_string());
        self.last_update_time = now;
    }

    pub fn get_favorites(&self) -> &Vec<String> {
        &self.favorites
    }

    pub fn get_last_update_time(&self) -> u64 {
        self.last_update_time
    }

    pub fn remove_favorite(&mut self, favorite: &str, now: u64) {
        self.favorites.retain(|f| f != favorite);
        self.last_update_time = now;
    }

    pub fn is_empty(&self) -> bool {
        self.favorites.is_empty()
    }
    pub fn len(&self) -> usize {
        self.favorites.len()
    }
}

#[derive(Default)]
pub(crate) struct UserFavoriteStore {
    user_favorites: HashMap<Principal, UserFavorite>,
}

impl UserFavoriteStore {
    pub fn new() -> Self {
        UserFavoriteStore {
            user_favorites: HashMap::new(),
        }
    }

    pub fn get_favorites(&self) -> &HashMap<Principal, UserFavorite> {
        &self.user_favorites
    }

    pub fn get_user_favorites(&self, user: &Principal) -> Option<&Vec<String>> {
        if let Some(user_favorites) = self.user_favorites.get(user) {
            Some(&user_favorites.favorites)
        } else {
            None
        }
    }

    pub fn add_favorite(&mut self, user: &Principal, now: u64, favorite: &str) {
        let user_favorite = self
            .user_favorites
            .entry(user.to_owned())
            .or_insert_with(|| UserFavorite::new());
        user_favorite.add_favorite(favorite, now);
    }

    pub fn remove_favorite(&mut self, user: &Principal, now: u64, favorite: &str) {
        debug!("remove favorite {}", favorite);
        let items = self.user_favorites.get_mut(&user.to_owned());
        if let Some(items) = items {
            items.remove_favorite(favorite, now);
            if items.is_empty() {
                debug!("remove user favorite");
                self.user_favorites.remove(&user);
            }
        }
    }
}

impl StableState for UserFavoriteStore {
    fn encode(&self) -> Vec<u8> {
        encode_args((&self.user_favorites, )).unwrap()
    }

    fn decode(bytes: Vec<u8>) -> Result<Self, String> {
        #[allow(clippy::type_complexity)]
            let (user_favorites, ): (HashMap<Principal, UserFavorite>, ) = decode_args(&bytes).unwrap();

        Ok(UserFavoriteStore { user_favorites })
    }
}

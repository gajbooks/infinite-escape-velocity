/*
    This file is part of Infinite Escape Velocity.

    Infinite Escape Velocity is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Infinite Escape Velocity is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Infinite Escape Velocity.  If not, see <https://www.gnu.org/licenses/>.
*/

const PROFILE_CLEANUP_DURATION: u64 = 1;

use std::{
    collections::HashMap,
    sync::{Arc, Weak},
    time::Duration,
};

use uuid::Uuid;

use crate::connectivity::player_info::player_profile::{AuthType, PlayerProfile};

#[derive(Clone)]
pub struct PlayerProfiles {
    player_list: Arc<tokio::sync::RwLock<HashMap<String, Arc<PlayerProfile>>>>,
}

impl PlayerProfiles {
    pub fn new() -> PlayerProfiles {
        let list = Arc::default();
        tokio::spawn(Self::cleanup_profiles_task(Arc::downgrade(&list)));
        PlayerProfiles { player_list: list }
    }

    pub async fn create_player_with_token(&self) -> Result<String, ()> {
        let id = Uuid::new_v4();
        self.create_player(AuthType::BasicToken {
            token: id.to_string(),
        })
        .await
    }

    pub async fn create_player_with_username_and_password(
        &self,
        desired_username: &str,
        desired_password: &str,
    ) -> Result<String, ()> {
        self.create_player(AuthType::UsernameAndPassword {
            username: desired_username.to_string(),
            password: desired_password.to_string(),
        })
        .await
    }

    pub async fn create_player(&self, auth: AuthType) -> Result<String, ()> {
        let identifier = PlayerProfiles::extract_identifier(&auth).to_string();

        let mut player_list = self.player_list.write().await;
        match player_list.entry(identifier.to_string()) {
            std::collections::hash_map::Entry::Occupied(_already_exists) => Err(()),
            std::collections::hash_map::Entry::Vacant(empty) => {
                empty.insert(Arc::new(PlayerProfile::new(auth)));
                Ok(identifier)
            }
        }
    }

    fn extract_identifier(auth: &AuthType) -> &str {
        match &auth {
            AuthType::BasicToken { token } => token,
            AuthType::UsernameAndPassword {
                username,
                password: _,
            } => username,
        }
    }

    pub async fn validate_login_request(&self, auth: &AuthType) -> Result<Arc<PlayerProfile>, ()> {
        let identifier = PlayerProfiles::extract_identifier(auth);

        let player_list = self.player_list.read().await;
        match player_list.get(identifier) {
            Some(valid) => {
                if valid.authentication == *auth {
                    Ok(valid.clone())
                } else {
                    Err(())
                }
            }
            None => Err(()),
        }
    }

    async fn cleanup_profiles_task(
        state: Weak<tokio::sync::RwLock<HashMap<String, Arc<PlayerProfile>>>>,
    ) {
        loop {
            tokio::time::sleep(Duration::from_secs(PROFILE_CLEANUP_DURATION)).await;
            if let Some(exists) = state.upgrade() {
                for (_identifier, profile) in &*exists.read().await {
                    profile.cleanup_expired_sessions();
                }
            } else {
                return;
            }
        }
    }
}

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

use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};

use uuid::Uuid;

use crate::connectivity::player_info::{
    player_profile::PlayerProfile, player_session::PlayerSession,
};

#[derive(Default, Clone)]
pub struct PlayerSessions {
    player_logins: Arc<tokio::sync::Mutex<HashMap<String, Weak<PlayerSession>>>>,
}

impl PlayerSessions {
    pub async fn get_session(&self, token: &str) -> Weak<PlayerSession> {
        let mut session_table = self.player_logins.lock().await;

        match session_table.entry(token.to_string()) {
            std::collections::hash_map::Entry::Occupied(session_cached_lookup) => {
                let session = session_cached_lookup.get().clone();

                if session.strong_count() == 0 {
                    session_cached_lookup.remove();
                }

                session
            }
            std::collections::hash_map::Entry::Vacant(_) => Weak::new(),
        }
    }

    pub async fn create_session(&self, profile: Arc<PlayerProfile>) -> String {
        let mut session_table = self.player_logins.lock().await;

        // Clean up old entries
        session_table.retain(|_id, session| session.upgrade().is_some());

        let mut session_id = Uuid::new_v4().to_string();

        while session_table.contains_key(&session_id) {
            session_id = Uuid::new_v4().to_string();
        }

        let session_weak_ref = match profile.session.get_session().upgrade() {
            Some(good_existing) => Arc::downgrade(&good_existing),
            None => profile.session.set_session(PlayerSession::new(profile.clone(), session_id.clone())),
        };

        match session_table.entry(session_id.clone()) {
            std::collections::hash_map::Entry::Occupied(_session_exists) => {
                // Session name already exists, which should never happen with a locked HashMap that we just checked
                unreachable!("Session identifiers should be pre-checked")
            }
            std::collections::hash_map::Entry::Vacant(add_session) => {
                add_session.insert(session_weak_ref);
                session_id
            }
        }
    }
}

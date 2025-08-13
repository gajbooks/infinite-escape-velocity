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

use std::{collections::HashMap, sync::Arc};

use uuid::Uuid;

use crate::backend::player_info::player_session::{PlayerSession, PlayerSessionTimeout};

#[derive(Default, Clone)]
pub struct PlayerSessions {
    player_logins: Arc<tokio::sync::Mutex<HashMap<String, Arc<PlayerSessionTimeout>>>>
}

impl PlayerSessions {
    pub async fn get_session(&self, token: &str) -> Option<Arc<PlayerSessionTimeout>> {
        let mut session_table = self.player_logins.lock().await;
        match session_table.entry(token.to_string()) {
            std::collections::hash_map::Entry::Occupied(session_exists) => {
                let session = session_exists.get();
                match session.is_within_session_async().await {
                    true => Some(session_exists.get().clone()),
                    false => {
                        session_exists.remove();
                        None
                    },
                }
            },
            std::collections::hash_map::Entry::Vacant(_) => None,
        }
    }

    pub async fn add_session(&self, session: PlayerSession) -> String {
        let mut session_table = self.player_logins.lock().await;

        let mut session_id = Uuid::new_v4().to_string();

        while session_table.contains_key(&session_id) {
            session_id = Uuid::new_v4().to_string();
        }

        match session_table.entry(session_id.clone()) {
            std::collections::hash_map::Entry::Occupied(_session_exists) => {
                // Session name already exists, which should never happen with a locked HashMap that we just checked
                unreachable!("Session identifiers should be pre-checked")
            },
            std::collections::hash_map::Entry::Vacant(add_session) => {
                add_session.insert(Arc::new(PlayerSessionTimeout::new(session)));
                session_id
            },
        }
    }
}
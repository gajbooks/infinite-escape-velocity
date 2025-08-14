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

use std::{sync::Arc, time::Instant};

use tokio::sync::{Mutex, MutexGuard};

use super::player_profile::PlayerProfile;

type InternalSessionType = Arc<tokio::sync::Mutex<PlayerSession>>;

const SESSION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(300);

pub struct PlayerSessionTimeout {
    last_connected_time: Mutex<Instant>,
    player_session: PlayerSession,
}

impl PlayerSessionTimeout {
    pub fn new(session: PlayerSession) -> Self {
        Self {last_connected_time: Mutex::new(std::time::Instant::now()), player_session: session}
    }

    pub async fn is_within_session_async(&self) -> bool {
        let locked_instant = self.last_connected_time.lock().await;
        Self::is_within_session_check(&locked_instant)
    }

    pub fn is_within_session(&self) -> bool {
        let locked_instant = self.last_connected_time.blocking_lock();
        Self::is_within_session_check(&locked_instant)
    }

    pub async fn get_session_async(&self) -> Option<&PlayerSession> {
        let mut locked_instant = self.last_connected_time.lock().await;
        self.get_session_logic(&mut locked_instant)
    }

    pub fn get_session(&self) -> Option<&PlayerSession> {
        let mut locked_instant = self.last_connected_time.blocking_lock();
        self.get_session_logic(&mut locked_instant)
    }

    fn is_within_session_check(time: &std::time::Instant) -> bool {
        if time.elapsed() > SESSION_TIMEOUT {
            false
        } else {
            true
        }
    }

    fn get_session_logic<'a>(&self, guard: &mut MutexGuard<'a, Instant>) -> Option<&PlayerSession> {
        match Self::extend_session(guard) {
            Ok(_instant) => {
                Some(&self.player_session)
            },
            Err(()) => {
                None
            },
        }
    }

    fn extend_session<'a>(guard: &mut MutexGuard<'a, Instant>) -> Result<Instant, ()> {
        if Self::is_within_session_check(&guard) {
            **guard = std::time::Instant::now();
            Ok(**guard + SESSION_TIMEOUT)
        } else {
            Err(())
        }
    }
}

pub struct PlayerSession {
    pub player_profile: Arc<PlayerProfile>,
    pub session_id: String,
}

impl PlayerSession {
    pub fn new(profile: Arc<PlayerProfile>, id: String) -> Self {
        Self{player_profile: profile, session_id: id}
    }
}

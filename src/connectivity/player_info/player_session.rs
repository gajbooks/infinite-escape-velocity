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
    sync::{Arc, Mutex, MutexGuard, Weak},
    time::Instant,
};

use tokio::{sync::RwLock, time::sleep};
use tracing::trace;

use crate::{
    backend::data_objects::input_status::InputStatus,
    connectivity::{
        client_server_message::ClientServerMessage,
        handlers::websocket_handler::WebsocketConnection,
    },
};

use super::player_profile::PlayerProfile;

const SESSION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);
const INPUT_REDRIVE: std::time::Duration = std::time::Duration::from_millis(1);

struct PlayerSessionData {
    last_connected_time: Instant,
    player_session: Option<Arc<PlayerSession>>,
}

pub struct PlayerSessionTimeout {
    data: Mutex<PlayerSessionData>,
}

impl PlayerSessionTimeout {
    pub fn new(session: Option<PlayerSession>) -> Self {
        Self {
            data: Mutex::new(PlayerSessionData {
                last_connected_time: std::time::Instant::now(),
                player_session: session.map(|x| Arc::new(x)),
            }),
        }
    }

    pub fn get_session(&self) -> Weak<PlayerSession> {
        let locked_data = self.data.lock().unwrap();
        if let Some(session) = &locked_data.player_session {
            if Self::is_within_session_check(&locked_data.last_connected_time) {
                Arc::downgrade(&session)
            } else {
                Weak::new()
            }
        } else {
            Weak::new()
        }
    }

    pub fn set_session(&self, session: PlayerSession) -> Weak<PlayerSession> {
        let mut locked_data = self.data.lock().unwrap();
        let new_session_ref = Arc::new(session);
        locked_data.player_session = Some(new_session_ref.clone());
        Self::reset_session_timer(&mut locked_data);
        let downgrade = Arc::downgrade(&new_session_ref);

        tokio::spawn(process_input_messages(downgrade.clone()));
        downgrade
    }

    fn is_within_session_check(time: &std::time::Instant) -> bool {
        if time.elapsed() > SESSION_TIMEOUT {
            false
        } else {
            true
        }
    }

    pub fn retain_if_valid(&self) -> bool {
        let mut guard = self.data.lock().unwrap();
        Self::retain_if_valid_intern(&mut guard)
    }

    fn retain_if_valid_intern<'a>(guard: &mut MutexGuard<'a, PlayerSessionData>) -> bool {
        if Self::is_within_session_check(&guard.last_connected_time) {
            true
        } else {
            if let Some(session) = &guard.player_session {
                trace!("Cleaned up player session {}", session.session_id);
            }

            guard.player_session = None;
            false
        }
    }

    pub fn extend_session<'a>(&self) -> bool {
        let mut guard = self.data.lock().unwrap();
        if Self::retain_if_valid_intern(&mut guard) {
            Self::reset_session_timer(&mut guard);
            true
        } else {
            false
        }
    }

    fn reset_session_timer<'a>(guard: &mut MutexGuard<'a, PlayerSessionData>) -> Instant {
        guard.last_connected_time = std::time::Instant::now();
        guard.last_connected_time + SESSION_TIMEOUT
    }
}

pub async fn process_input_messages(profile: Weak<PlayerSession>) {
    'socket_poll: loop {
        sleep(INPUT_REDRIVE).await;

        if let Some(valid_session) = profile.upgrade() {
            if let Some(has_websocket) = &mut *valid_session.websocket_connection.lock().await {
                if has_websocket.cancel.is_canceled() {
                    continue 'socket_poll;
                }

                let mut input_status = valid_session.input_status.write().await;

                'message_poll: loop {
                    match has_websocket.inbound.try_recv() {
                        Ok(has_message) => {
                            // Extend current session through roundabout means
                            valid_session.player_profile.session.extend_session();

                            match has_message {
                                ClientServerMessage::Authorize { token: _ } => {
                                    // We don't want to handle authorize messages here, but we are required to send them over the websocket
                                }
                                ClientServerMessage::Disconnect => {
                                    let _ = has_websocket.cancel.cancel();
                                    continue 'socket_poll;
                                }
                                ClientServerMessage::ControlInput {
                                    input,
                                    pressed,
                                } => {
                                    match input {
                                    crate::connectivity::client_server_message::ControlInput::Forward => {
                                        input_status.forward = pressed;
                                    },
                                    crate::connectivity::client_server_message::ControlInput::Backward => {
                                        input_status.backward = pressed;
                                    },
                                    crate::connectivity::client_server_message::ControlInput::Left => {
                                        input_status.left = pressed;
                                    },
                                    crate::connectivity::client_server_message::ControlInput::Right => {
                                        input_status.right = pressed;
                                    },
                                    crate::connectivity::client_server_message::ControlInput::Fire => {
                                        input_status.fire = pressed;
                                    },
                                    }
                                }
                            }
                        }
                        Err(e) => match e {
                            tokio::sync::mpsc::error::TryRecvError::Empty => break 'message_poll,
                            tokio::sync::mpsc::error::TryRecvError::Disconnected => {
                                return;
                            }
                        },
                    }
                }
            } else {
                continue 'socket_poll;
            }
        } else {
            return;
        }
    }
}

pub struct PlayerSession {
    pub input_status: RwLock<InputStatus>,
    pub player_profile: Arc<PlayerProfile>,
    pub session_id: String,
    pub websocket_connection: tokio::sync::Mutex<Option<WebsocketConnection>>,
}

impl PlayerSession {
    pub fn new(profile: Arc<PlayerProfile>, session_id: String) -> Self {
        Self {
            input_status: InputStatus::default().into(),
            player_profile: profile,
            session_id,
            websocket_connection: None.into(),
        }
    }
}

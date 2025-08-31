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

const CONTROL_INPUT_MESSAGE_CAPACITY: usize = 100;

use bevy_ecs::{component::Component, entity::Entity, system::Query};
use tokio::sync::broadcast;

use crate::connectivity::{
    client_server_message::ClientServerMessage, client_server_message::ControlInput,
};

use std::sync::Weak;

use crate::connectivity::player_info::player_session::PlayerSession;

pub fn process_incoming_messages(user_sessions: Query<&mut PlayerSessionComponent>) {
    user_sessions.par_iter().for_each(|session| {
        let session_exists = match session.session.upgrade() {
            Some(session_valid) => session_valid,
            None => {
                // Session doesn't exist
                return;
            },
        };

        let mut websocket_connection = session_exists.websocket_connection.lock().unwrap();

        let websocket_connection = match &mut *websocket_connection {
            Some(connected) => connected,
            None => {
                // No websocket in session
                return;
            },
        };
        
        if websocket_connection.cancel.is_canceled() {
            return;
        }

        while let Ok(message) = websocket_connection.inbound.try_recv() {
            match message {
                ClientServerMessage::Authorize { token: _ } => {
                    // We don't want to handle authorize messages here, but we are required to send them over the websocket
                }
                ClientServerMessage::Disconnect => {
                    let _ = websocket_connection.cancel.cancel();
                    continue;
                }
                ClientServerMessage::ControlInput { input, pressed } => {
                    // We can't do anything about send errors when there are no receive handles
                    let _ = session
                        .control_input_sender
                        .send(ControlInputMessage { input, pressed });
                }
            }
        }
    });
}

#[derive(Clone)]
pub struct ControlInputMessage {
    pub input: ControlInput,
    pub pressed: bool,
}

#[derive(Component)]
pub struct PlayerSessionComponent {
    pub session: Weak<PlayerSession>,
    pub control_input_sender: broadcast::Sender<ControlInputMessage>,
    pub primary_viewport: Option<Entity>,
    pub should_follow: Option<Entity>,
}

impl PlayerSessionComponent {
    pub fn new(session: Weak<PlayerSession>) -> PlayerSessionComponent {
        let session = PlayerSessionComponent {
            session,
            control_input_sender: broadcast::Sender::new(CONTROL_INPUT_MESSAGE_CAPACITY),
            primary_viewport: None,
            should_follow: None,
        };
        session
    }
}
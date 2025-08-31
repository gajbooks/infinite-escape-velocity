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

use bevy_ecs::{component::Component, entity::Entity, system::Query};

use std::sync::Weak;

use crate::{
    backend::data_objects::input_status::InputStatus,
    connectivity::{
        client_server_message::ClientServerMessage, player_info::player_session::PlayerSession,
        server_client_message::ServerClientMessage,
    },
};

#[derive(Component)]
pub struct PlayerSessionComponent {
    pub command_queue_inbound: async_channel::Receiver<ClientServerMessage>,
    pub command_queue_outbound: async_channel::Sender<ServerClientMessage>,
    pub input_status: InputStatus,
    pub primary_viewport: Option<Entity>,
    pub session: Weak<PlayerSession>,
    pub should_follow: Option<Entity>,
}

impl PlayerSessionComponent {
    pub fn new(
        session: Weak<PlayerSession>,
        command_queue_inbound: async_channel::Receiver<ClientServerMessage>,
        command_queue_outbound: async_channel::Sender<ServerClientMessage>,
    ) -> PlayerSessionComponent {
        let session = PlayerSessionComponent {
            command_queue_inbound,
            command_queue_outbound,
            input_status: InputStatus::default(),
            session,
            primary_viewport: None,
            should_follow: None,
        };
        session
    }
}

pub fn process_input_messages_system(mut sessions: Query<&mut PlayerSessionComponent>) {
    sessions.par_iter_mut().for_each(|mut session| {
                    match session.command_queue_inbound.try_recv() {
                        Ok(has_message) => {
                            match has_message {
                                ClientServerMessage::Authorize { token: _ } => {
                                    // We don't want to handle authorize messages here, but we are required to send them over the websocket
                                }
                                ClientServerMessage::ControlInput {
                                    input,
                                    pressed,
                                } => {
                                    match input {
                                    crate::connectivity::client_server_message::ControlInput::Forward => {
                                        session.input_status.forward = pressed;
                                    },
                                    crate::connectivity::client_server_message::ControlInput::Backward => {
                                        session.input_status.backward = pressed;
                                    },
                                    crate::connectivity::client_server_message::ControlInput::Left => {
                                        session.input_status.left = pressed;
                                    },
                                    crate::connectivity::client_server_message::ControlInput::Right => {
                                        session.input_status.right = pressed;
                                    },
                                    crate::connectivity::client_server_message::ControlInput::Fire => {
                                        session.input_status.fire = pressed;
                                    },
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            match e {
                                async_channel::TryRecvError::Empty => return,
                                async_channel::TryRecvError::Closed => return,
                            }
                        },
                    }
    });
}

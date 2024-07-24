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

use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use bevy_ecs::{component::Component, entity::Entity, system::Query};
use tokio::sync::{broadcast, mpsc::{UnboundedReceiver, UnboundedSender}};

use crate::connectivity::{
    client_server_message::ClientServerMessage, server_client_message::ServerClientMessage,
};

use super::client_server_message::ControlInput;

pub fn process_incoming_messages(mut user_sessions: Query<&mut UserSession>) {
    user_sessions.par_iter_mut().for_each(|mut session| {
        if session.cancel.load(Ordering::Relaxed) == true {
            return;
        }

        while let Ok(message) = session.from_remote.try_recv() {
            match message {
                ClientServerMessage::Authorize(_) => {
                    // We don't want to handle authorize messages here, but we are required to send them over the websocket
                },
                ClientServerMessage::Disconnect => {
                    let _ = session.cancel.store(true, Ordering::Relaxed);
                    continue;
                },
                ClientServerMessage::ControlInput { input, pressed } => {
                    // We can't do anything about send errors when there are no receive handles
                    let _ = session.control_input_sender.send(ControlInputMessage{input, pressed});
                }
            }
        }
    });
}

#[derive(Clone)]
pub struct ControlInputMessage {
    pub input: ControlInput, 
    pub pressed: bool
}

#[derive(Component)]
pub struct UserSession {
    pub remote_address: SocketAddr,
    pub to_remote: UnboundedSender<ServerClientMessage>,
    from_remote: UnboundedReceiver<ClientServerMessage>,
    pub control_input_sender: broadcast::Sender<ControlInputMessage>,
    pub cancel: Arc<AtomicBool>,
    pub primary_viewport: Option<Entity>,
    pub should_follow: Option<Entity>,
}

impl UserSession {
    pub fn spawn_user_session(
        to_remote: UnboundedSender<ServerClientMessage>,
        from_remote: UnboundedReceiver<ClientServerMessage>,
        remote_address: SocketAddr,
        cancel: Arc<AtomicBool>,
    ) -> UserSession {
        let session = UserSession {
            from_remote,
            to_remote: to_remote.clone(),
            control_input_sender: broadcast::Sender::new(CONTROL_INPUT_MESSAGE_CAPACITY),
            remote_address: remote_address,
            cancel: cancel.clone(),
            primary_viewport: None,
            should_follow: None,
        };
        session
    }
}

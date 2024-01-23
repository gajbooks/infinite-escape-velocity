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
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use bevy_ecs::{component::Component, entity::Entity};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::connectivity::{
    client_server_message::ClientServerMessage, server_client_message::ServerClientMessage,
};

#[derive(Component)]
pub struct UserSession {
    pub remote_address: SocketAddr,
    pub to_remote: UnboundedSender<ServerClientMessage>,
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
            to_remote: to_remote.clone(),
            remote_address: remote_address,
            cancel: cancel.clone(),
            primary_viewport: None,
            should_follow: None,
        };
        tokio::spawn(Self::process_incoming_messages(
            cancel,
            to_remote,
            from_remote,
        ));
        session
    }

    async fn process_incoming_messages(
        cancel: Arc<AtomicBool>,
        _to_remote: UnboundedSender<ServerClientMessage>,
        mut from_remote: UnboundedReceiver<ClientServerMessage>,
    ) {
        loop {
            if cancel.load(Ordering::Relaxed) == true {
                break;
            }

            let message = from_remote.recv().await;

            let message = match message {
                Some(x) => x,
                None => {
                    continue;
                }
            };

            match message {
                ClientServerMessage::Disconnect => {
                    let _ = cancel.store(true, Ordering::Relaxed);
                    continue;
                }
                _ => (),
            }
        }
    }
}

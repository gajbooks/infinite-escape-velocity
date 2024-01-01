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
        Arc, Mutex,
    },
};

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{
    backend::shape::{CircleData, Shape},
    connectivity::{
        client_server_message::ClientServerMessage, server_client_message::ServerClientMessage,
    },
    shared_types::{Coordinates, Radius},
};

pub struct UserSession {
    pub remote_address: SocketAddr,
    pub to_remote: UnboundedSender<ServerClientMessage>,
    pub viewports_to_spawn: Mutex<Vec<Shape>>,
    pub cancel: Arc<AtomicBool>,
    dead: AtomicBool,
}

impl UserSession {
    pub fn spawn_user_session(
        to_remote: UnboundedSender<ServerClientMessage>,
        from_remote: UnboundedReceiver<ClientServerMessage>,
        remote_address: SocketAddr,
        cancel: Arc<AtomicBool>,
    ) -> Arc<UserSession> {
        let session = Arc::new(UserSession {
            to_remote: to_remote.clone(),
            remote_address: remote_address,
            cancel: cancel,
            dead: false.into(),
            viewports_to_spawn: Mutex::new(vec![Shape::Circle(CircleData {
                radius: Radius::new(600.0),
                location: Coordinates::new(0.0, 0.0),
            })]),
        });
        tokio::spawn(
            session
                .clone()
                .process_incoming_messages(to_remote, from_remote),
        );
        session
    }

    async fn process_incoming_messages(
        self: Arc<Self>,
        _to_remote: UnboundedSender<ServerClientMessage>,
        mut from_remote: UnboundedReceiver<ClientServerMessage>,
    ) {
        loop {
            if self.cancel.load(Ordering::Relaxed) == true {
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
                    self.disconnect();
                    continue;
                }
                _ => (),
            }
        }

        self.mark_dead();
    }

    fn mark_dead(&self) {
        self.dead.store(true, Ordering::Relaxed);
    }

    pub fn is_dead(&self) -> bool {
        self.dead.load(Ordering::Relaxed)
    }

    pub fn disconnect(&self) {
        let _ = self.cancel.store(true, Ordering::Relaxed);
    }
}

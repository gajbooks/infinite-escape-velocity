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

use crate::connectivity::client_server_message::*;
use crate::connectivity::player_info::player_sessions::PlayerSessions;
use crate::connectivity::server_client_message::*;
use crate::connectivity::user_session::*;
use crate::utility::cancel_flag::CancelFlag;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use bytes::Bytes;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::timeout;

use futures::stream::StreamExt;
use futures_util::SinkExt;
use std::net::SocketAddr;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::mpsc::unbounded_channel;
use tracing::debug;
use tracing::info;

const WEBSOCKET_TIMEOUT: Duration = Duration::from_secs(1);
const AUTHORIZATION_TIMEOUT: Duration = Duration::from_secs(5);

pub struct WebsocketConnection {
    pub cancel: CancelFlag,
    pub inbound: UnboundedReceiver<ClientServerMessage>,
    pub outbound: UnboundedSender<ServerClientMessage>,
    pub remote_address: SocketAddr,
}

impl Drop for WebsocketConnection {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

#[derive(Clone)]
pub struct HandlerState {
    pub connections: UnboundedSender<UserSession>,
    pub sessions: PlayerSessions,
}

pub async fn websocket_handler(
    websocket: WebSocketUpgrade,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    State(state): State<HandlerState>,
) -> impl IntoResponse {
    tracing::trace!("{address} attempted WebSocket upgrade.");
    websocket.on_upgrade(move |socket| handle_socket(socket, address, state))
}

async fn handle_socket(socket: WebSocket, who: SocketAddr, state: HandlerState) {
    let (mut sender, mut receiver) = socket.split();

    let (outbound_messages_sender, mut outbound_messages_receiver) =
        unbounded_channel::<ServerClientMessage>();
    let (inbound_messages_sender, inbound_messages_receiver) =
        unbounded_channel::<ClientServerMessage>();
    let canceled = CancelFlag::default();

    let external_task_cancel = canceled.clone();
    let outbound_task_cancel = canceled.clone();
    let inbound_task_cancel = canceled;

    let connection = WebsocketConnection {
        outbound: outbound_messages_sender,
        inbound: inbound_messages_receiver,
        cancel: external_task_cancel,
        remote_address: who,
    };

    let outbound_task = tokio::spawn(async move {
        loop {
            match outbound_task_cancel.is_canceled() {
                true => {
                    tracing::trace!("Outbound task received stop signal for {}", who);
                    let _ = sender.send(Message::Close(None)).await; // We don't mind if sender fails to send close frames
                    break;
                }
                false => {}
            }

            let message_to_send = outbound_messages_receiver.recv().await;
            match message_to_send {
                Some(outgoing_message) => {
                    let mut serialized = Vec::<u8>::new();
                    // It would be very difficult for a serialization to fail, and would likely be a programming issue on the server
                    ciborium::into_writer(&outgoing_message, &mut serialized).unwrap();

                    if sender
                        .send(Message::Binary(Bytes::from(serialized)))
                        .await
                        .is_err()
                    {
                        tracing::warn!("Websocket send failed to {}", who);
                        outbound_task_cancel.cancel();
                    }
                }
                None => {
                    // All senders closed, ignoring case of external closure
                    if outbound_task_cancel.cancel() {
                        tracing::trace!(
                            "Outbound task received cancel signal but was waiting in message send when socket was dropped for {}",
                            who
                        );
                    } else {
                        tracing::info!("Server dropped connection to {}", who);
                    }
                }
            }
        }

        tracing::trace!("Outbound task exiting for {}", who);
    });

    let inbound_task = tokio::spawn(async move {
        loop {
            match inbound_task_cancel.is_canceled() {
                true => {
                    tracing::trace!("Inbound task received stop signal for {}", who);
                    break;
                }
                false => {}
            }

            match timeout(WEBSOCKET_TIMEOUT, receiver.next()).await {
                Ok(has_message) => {
                    match has_message {
                        Some(socket_message) => {
                            match socket_message {
                                Ok(incoming) => {
                                    match incoming {
                                        Message::Binary(bin) => {
                                            match ciborium::from_reader(&*bin) {
                                                Ok(deserialized) => {
                                                    match inbound_messages_sender.send(deserialized)
                                                    {
                                                        Ok(()) => (),
                                                        Err(_) => {
                                                            // Internal sender disconnected, abort
                                                            tracing::warn!(
                                                                "Server disconnected inbound messages from: {}",
                                                                who
                                                            );
                                                            inbound_task_cancel.cancel();
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    // Message couldn't be deserialized for some reason. Not fatal but it means something is wrong.
                                                    tracing::warn!(
                                                        "Nonsense undeserializable websocket message {:?} received from {}",
                                                        e,
                                                        who
                                                    );
                                                }
                                            }
                                        }
                                        Message::Close(_) => {
                                            // User disconnected gracefully
                                            tracing::info!("User at {} disconnected", who);
                                            inbound_task_cancel.cancel();
                                        }
                                        _ => {
                                            // What are you, some kind of wandering butler or something?
                                        }
                                    }
                                }
                                Err(e) => {
                                    // Message receive error
                                    tracing::warn!(
                                        "User at {} disconnected with error {:?}",
                                        who,
                                        e
                                    );
                                    inbound_task_cancel.cancel();
                                }
                            }
                        }
                        None => {
                            // Rx stream is dead somehow without getting close messages
                            tracing::warn!("User at {} disconnected ungracefully", who);
                            inbound_task_cancel.cancel();
                        }
                    }
                }
                Err(_elapsed) => {
                    // Websocket will re-check if it should cancel and drop, but this does not mean the entire websocket has failed
                }
            }
        }

        tracing::trace!("Inbound task exiting for {}", who);
    });

    tokio::spawn(wait_for_websocket_login_message(state, connection));

    let _ = tokio::join!(outbound_task, inbound_task);
    tracing::trace!("Websocket connection task finished for {}", who);
}

async fn wait_for_websocket_login_message(
    state: HandlerState,
    mut connection: WebsocketConnection,
) {
    let auth_start = Instant::now();
    while let Ok(Some(message)) = timeout(AUTHORIZATION_TIMEOUT, connection.inbound.recv()).await {
        if auth_start.elapsed() > AUTHORIZATION_TIMEOUT {
            info!(
                "Websocket authorization timeout exceeded from {}",
                connection.remote_address
            );
            return;
        }

        match message {
            ClientServerMessage::Authorize { token } => {
                match state.sessions.get_session(&token).await.upgrade() {
                    Some(valid_session) => {
                        // TODO: Actually put this in the session somewhere?
                        info!(
                            "Authorized websocket connection using client server message from {}",
                            connection.remote_address
                        );

                        state
                            .connections
                            .send(UserSession::spawn_user_session(connection))
                            .unwrap(); // Can't do anything if the other portion is disconnected

                        return;
                    }
                    None => {
                        info!(
                            "User websocket login failed with attempted token {} from {}",
                            &token, connection.remote_address
                        );
                        return;
                    }
                }
            }
            ClientServerMessage::Disconnect => {
                debug!(
                    "User disconnected before authorizing websocket from {}",
                    connection.remote_address
                );
                return;
            }
            _ => (), // Don't handle anything else
        }
    }

    debug!(
        "User closed socket before authorizing websocket from {}",
        connection.remote_address
    );
}

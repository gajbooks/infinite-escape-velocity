use crate::connectivity::client_server_message::*;
use crate::connectivity::connected_users::*;
use crate::connectivity::server_client_message::*;
use crate::connectivity::user_session::*;
use axum::extract::ws::CloseFrame;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use tokio::time::timeout;

use futures::stream::StreamExt;
use futures_util::SinkExt;
use std::borrow::Cow;
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;
use std::sync::*;
use std::time::Duration;
use tokio::sync::mpsc::unbounded_channel;

pub async fn websocket_handler(
    websocket: WebSocketUpgrade,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    State(connections): State<Arc<ConnectedUsers>>,
) -> impl IntoResponse {
    tracing::trace!("{address} attempted WebSocket upgrade.");
    websocket.on_upgrade(move |socket| handle_socket(socket, address, connections))
}

async fn handle_socket(socket: WebSocket, who: SocketAddr, connections: Arc<ConnectedUsers>) {
    let (mut sender, mut receiver) = socket.split();

    let (outbound_messages_sender, mut outbound_messages_receiver) =
        unbounded_channel::<ServerClientMessage>();
    let (inbound_messages_sender, inbound_messages_receiver) =
        unbounded_channel::<ClientServerMessage>();
    let canceled = Arc::new(AtomicBool::new(false));

    let external_task_cancel = canceled.clone();
    let outbound_task_cancel = canceled.clone();
    let inbound_task_cancel = canceled;

    connections.add_user(UserSession::new(
        outbound_messages_sender,
        inbound_messages_receiver,
        who,
        external_task_cancel,
    ));

    let outbound_task = tokio::spawn(async move {
        loop {
            match outbound_task_cancel.load(atomic::Ordering::Relaxed) {
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
                    // It would be very difficult for a Serde serialization to fail, and would likely be a programming issue on the server
                    let serialized = serde_json::to_string(&outgoing_message).unwrap();

                    if sender.send(Message::Text(serialized)).await.is_err() {
                        tracing::warn!("Websocket send failed to {}", who);
                        outbound_task_cancel.store(true, atomic::Ordering::Relaxed);
                    }
                }
                None => {
                    // All senders closed, ignoring case of external closure
                    if outbound_task_cancel.load(atomic::Ordering::Relaxed) {
                        tracing::trace!("Outbound task received cancel signal but was waiting in message send when socket was dropped for {}", who);
                    } else {
                        tracing::info!("Server dropped connection to {}", who);
                    }

                    outbound_task_cancel.store(true, atomic::Ordering::Relaxed);
                }
            }
        }

        tracing::trace!("Outbound task exiting for {}", who);
    });

    let inbound_task = tokio::spawn(async move {
        loop {
            match inbound_task_cancel.load(atomic::Ordering::Relaxed) {
                true => {
                    tracing::trace!("Inbound task received stop signal for {}", who);
                    break;
                }
                false => {}
            }

            match timeout(Duration::from_secs(1), receiver.next()).await {
                Ok(has_message) => {
                    match has_message {
                        Some(socket_message) => {
                            match socket_message {
                                Ok(incoming) => {
                                    match incoming {
                                        Message::Text(message) => {
                                            match serde_json::from_str(&message) {
                                                Ok(deserialized) => {
                                                    match inbound_messages_sender.send(deserialized)
                                                    {
                                                        Ok(()) => (),
                                                        Err(_) => {
                                                            // Internal sender disconnected, abort
                                                            tracing::warn!("Server disconnected inbound messages from: {}", who);
                                                            inbound_task_cancel.store(
                                                                true,
                                                                atomic::Ordering::Relaxed,
                                                            );
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    // Message couldn't be deserialized for some reason. Not fatal but it means something is wrong.
                                                    tracing::warn!("Nonsense undeserializable websocket message {:?} received from {}", e, who);
                                                }
                                            }
                                        }
                                        Message::Close(_) => {
                                            // User disconnected gracefully
                                            tracing::info!("User at {} disconnected", who);
                                            inbound_task_cancel
                                                .store(true, atomic::Ordering::Relaxed);
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
                                    inbound_task_cancel.store(true, atomic::Ordering::Relaxed);
                                }
                            }
                        }
                        None => {
                            // Rx stream is dead somehow without getting close messages
                            tracing::warn!("User at {} disconnected ungracefully", who);
                            inbound_task_cancel.store(true, atomic::Ordering::Relaxed);
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

    let _ = tokio::join!(outbound_task, inbound_task);
    tracing::trace!("Websocket connection task finished for {}", who);
}

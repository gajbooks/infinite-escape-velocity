use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use tokio::sync::broadcast;
use crate::connectivity::client_server_message::*;
use crate::connectivity::server_client_message::*;
use crate::connectivity::connected_users::*;
use crate::connectivity::user_session::*;

use futures::stream::StreamExt;
use futures_util::SinkExt;
use std::net::SocketAddr;
use std::sync::*;
use tokio::sync::broadcast::error::TryRecvError;
use tokio::sync::mpsc::unbounded_channel;

pub async fn websocket_handler(
    websocket: WebSocketUpgrade,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    State(connections): State<Arc<ConnectedUsers>>,
) -> impl IntoResponse {
    println!("{address} connected.");
    websocket.on_upgrade(move |socket| handle_socket(socket, address, connections))
}

async fn handle_socket(socket: WebSocket, who: SocketAddr, connections: Arc<ConnectedUsers>) {
    let (mut sender, mut receiver) = socket.split();

    let (outbound_messages_sender, mut outbound_messages_receiver) =
        unbounded_channel::<ServerClientMessage>();
    let (inbound_messages_sender, inbound_messages_receiver) =
        unbounded_channel::<ClientServerMessage>();
    let (cancel_messages_sender, cancel_messages_receiver) = broadcast::channel::<()>(1);

    let mut outbound_task_cancel_messages_receiver = cancel_messages_receiver.resubscribe();
    let mut inbound_task_cancel_messages_receiver = cancel_messages_receiver;

    let external_task_cancel_messages_sender = cancel_messages_sender.clone();
    let outbound_task_cancel_messages_sender = cancel_messages_sender.clone();
    let inbound_task_cancel_messages_sender = cancel_messages_sender;

    connections.add_user(UserSession::new(
        outbound_messages_sender,
        inbound_messages_receiver,
        who,
        external_task_cancel_messages_sender,
    ));

    let outbound_task = tokio::spawn(async move {
        loop {
            match outbound_task_cancel_messages_receiver.try_recv() {
                Ok(()) => {
                    break;
                }
                Err(e) => match e {
                    TryRecvError::Empty => {}
                    _ => {
                        break;
                    }
                },
            }

            let message_to_send = outbound_messages_receiver.recv().await;
            match message_to_send {
                Some(outgoing_message) => {
                    // It would be very difficult for a Serde serialization to fail, and would likely be a programming issue on the server
                    let serialized = serde_json::to_string(&outgoing_message).unwrap();

                    if sender.send(Message::Text(serialized)).await.is_err() {
                        println!("Websocket send failed to {}", who);
                        let _ = outbound_task_cancel_messages_sender.send(());
                    }
                }
                None => {
                    // All senders closed
                    println!("Server dropped connection to {}", who);
                    let _ = outbound_task_cancel_messages_sender.send(());
                }
            }
        }
    });

    let inbound_task = tokio::spawn(async move {
        loop {
            match inbound_task_cancel_messages_receiver.try_recv() {
                Ok(()) => {
                    break;
                }
                Err(e) => match e {
                    TryRecvError::Empty => {}
                    _ => {
                        break;
                    }
                },
            }

            match receiver.next().await {
                Some(socket_message) => {
                    match socket_message {
                        Ok(incoming) => {
                            match incoming {
                                Message::Text(message) => {
                                    match serde_json::from_str(&message) {
                                        Ok(deserialized) => {
                                            match inbound_messages_sender.send(deserialized) {
                                                Ok(()) => (),
                                                Err(_) => {
                                                    // Internal sender disconnected, abort
                                                    println!("Server disconnected inbound messages from: {}", who);
                                                    let _ = inbound_task_cancel_messages_sender
                                                        .send(());
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            // Message couldn't be deserialized for some reason. Not fatal but it means something is wrong.
                                            println!("Nonsense undeserializable websocket message {:?} received from {}", e, who);
                                        }
                                    }
                                }
                                Message::Close(_) => {
                                    // User disconnected gracefully
                                    println!("User at {} disconnected", who);
                                    let _ = inbound_task_cancel_messages_sender.send(());
                                }
                                _ => {
                                    // What are you, some kind of wandering butler or something?
                                }
                            }
                        }
                        Err(e) => {
                            // Message receive error
                            println!("User at {} disconnected with error {:?}", who, e);
                            let _ = inbound_task_cancel_messages_sender.send(());
                        }
                    }
                }
                None => {
                    // Rx stream is dead somehow without getting close messages
                    println!("User at {} disconnected ungracefully", who);
                    let _ = inbound_task_cancel_messages_sender.send(());
                }
            }
        }
    });

    let _ = tokio::join!(outbound_task, inbound_task);
}
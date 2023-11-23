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

mod backend;
mod configuration_loaders;
mod connectivity;
mod shared_types;

use axum::extract::ws::{WebSocket, Message};
use axum::extract::{ConnectInfo, WebSocketUpgrade, State};
use axum::http::Response;
use axum::response::IntoResponse;
use axum::{
    routing::get,
    Router,
};
use clap::Parser;
use connectivity::client_server_message::*;
use connectivity::server_client_message::*;

use futures_util::SinkExt;
use tokio::sync::broadcast::error::TryRecvError;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use tokio::sync::mpsc::UnboundedSender;
use tokio::time;
use tower_http::trace::{TraceLayer, DefaultMakeSpan};
use std::time::Duration;
use futures::stream::StreamExt;
use std::net::SocketAddr;
use std::sync::*;

use tokio::sync::broadcast;

use tower_http::services::ServeDir;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Directory to host the webapp from. If ommitted, server is started in dedicated mode.
    #[arg(long)]
    webapp_directory: Option<String>,

    /// Directory to load gamedata from.
    data_directory: String
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let app = Router::new();

    let app = match &args.webapp_directory {
        Some(webapp_directory) => {
            app.nest_service("/", ServeDir::new(webapp_directory))
        },
        None => {
            app
        }
    };

    let user_connections = Arc::new(ConnectedUsers{connection_list: Mutex::new(Vec::new())});

    tokio::spawn(ConnectedUsers::bring_out_your_dead(user_connections.clone()));

    let app = app.route("/ws", get(websocket_handler)).with_state(user_connections.clone());
    
    axum::Server::bind(&"0.0.0.0:2718".parse().unwrap())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

async fn websocket_handler(
    websocket: WebSocketUpgrade,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    State(connections): State<Arc<ConnectedUsers>>
) -> impl IntoResponse {
    println!("{address} connected.");
    websocket.on_upgrade(move |socket| handle_socket(socket, address, connections))
}

async fn handle_socket(socket: WebSocket, who: SocketAddr, connections: Arc<ConnectedUsers>) {
    let (mut sender, mut receiver) = socket.split();

    let (outbound_messages_sender, mut outbound_messages_receiver) = unbounded_channel::<ServerClientMessage>();
    let (inbound_messages_sender, inbound_messages_receiver) = unbounded_channel::<ClientServerMessage>();
    let (cancel_messages_sender, cancel_messages_receiver) = broadcast::channel::<()>(1);

    let mut outbound_task_cancel_messages_receiver = cancel_messages_receiver.resubscribe();
    let mut inbound_task_cancel_messages_receiver = cancel_messages_receiver;

    let external_task_cancel_messages_sender = cancel_messages_sender.clone();
    let outbound_task_cancel_messages_sender = cancel_messages_sender.clone();
    let inbound_task_cancel_messages_sender = cancel_messages_sender;

    connections.add_user(UserSession { to_remote: outbound_messages_sender, from_remote: inbound_messages_receiver, remote_address: who, cancel: external_task_cancel_messages_sender });

    let outbound_task = tokio::spawn(
        async move {
            loop {
                match outbound_task_cancel_messages_receiver.try_recv() {
                    Ok(()) => {
                        break;
                    }
                    Err(e) => {
                        match e {
                            TryRecvError::Empty => {

                            },
                            _ => {
                                break;
                            }
                        }
                    }
                }
                
                let message_to_send = outbound_messages_receiver.recv().await;
                match message_to_send {
                    Some(outgoing_message) => {
                        // It would be very difficult for a Serde serialization to fail, and would likely be a programming issue on the server
                        let serialized = serde_json::to_string(&outgoing_message).unwrap();
                        
                        if sender.send(Message::Text(serialized)).await.is_err()
                        {
                            println!("Websocket send failed to {}", who);
                            let _ = outbound_task_cancel_messages_sender.send(());
                        }
                    },
                    None => {
                        // All senders closed
                        println!("Server dropped connection to {}", who);
                        let _ = outbound_task_cancel_messages_sender.send(());
                    }
                }
            }
        }
    );

    let inbound_task = tokio::spawn(
        async move {
            loop {
                match inbound_task_cancel_messages_receiver.try_recv() {
                    Ok(()) => {
                        break;
                    }
                    Err(e) => {
                        match e {
                            TryRecvError::Empty => {

                            },
                            _ => {
                                break;
                            }
                        }
                    }
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
                                                        let _ = inbound_task_cancel_messages_sender.send(());
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
        }
    );

    let _ = tokio::join!(outbound_task, inbound_task);
}
struct UserSession {
    to_remote: UnboundedSender<ServerClientMessage>,
    from_remote: UnboundedReceiver<ClientServerMessage>,
    remote_address: SocketAddr,
    cancel: broadcast::Sender<()>
}

impl UserSession {
    pub fn is_dead(&self) -> bool {
        self.to_remote.is_closed()
    }

    pub fn disconnect(&self) {
        self.cancel.send(());
    }
}

struct ConnectedUsers {
    connection_list: Mutex<Vec<UserSession>>
}

impl ConnectedUsers {
    pub fn add_user(&self, new_user: UserSession)
    {
        let mut connected_users = self.connection_list.lock().unwrap();
        println!("New user connected from {}", new_user.remote_address);
        connected_users.push(new_user);
    }

    pub async fn bring_out_your_dead(connected_users: Arc<ConnectedUsers>) {
            let mut interval = time::interval(Duration::from_millis(500));

            loop {
                interval.tick().await;
                {
                    let mut list = connected_users.connection_list.lock().unwrap();
                    list.retain(|x| !x.is_dead());
                }
            }
    }
}
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

use std::convert::Infallible;
use std::time::Duration;

use axum::Json;
use axum::http::HeaderMap;
use axum::response::Sse;
use axum::response::sse::{Event, KeepAlive};
use axum::{extract::State, http::StatusCode};
use futures::{Stream, stream};
use tokio::time::timeout;
use tracing::warn;

use crate::connectivity::models::chat_message_request::ChatMessageRequest;
use crate::connectivity::player_info::player_sessions::PlayerSessions;
use crate::connectivity::services::chat_service::ChatService;

const MAX_MESSAGE_LENGTH: usize = 2048;
const MESSAGE_REDRIVE_TIMEOUT_SECONDS: u64 = 10;

pub async fn send_message(
    State((chat_service, player_sessions)): State<(ChatService, PlayerSessions)>,
    headers: HeaderMap,
    Json(message): Json<ChatMessageRequest>,
) -> StatusCode {
    if message.message.len() > MAX_MESSAGE_LENGTH {
        return StatusCode::BAD_REQUEST;
    }

    match headers.get("Authorization") {
        Some(auth_header) => match auth_header.to_str() {
            Ok(auth_header_string) => match player_sessions
                .get_session(auth_header_string)
                .await
                .upgrade()
            {
                Some(existing_session) => {
                    let username = existing_session
                        .player_profile
                        .authentication
                        .get_username();
                    chat_service.send_message(&message.message, username);
                    StatusCode::NO_CONTENT
                }
                None => StatusCode::UNAUTHORIZED,
            },
            Err(_) => StatusCode::UNAUTHORIZED,
        },
        None => StatusCode::UNAUTHORIZED,
    }
}

pub async fn subscribe_message(
    State((chat_service, player_sessions)): State<(ChatService, PlayerSessions)>,
    headers: HeaderMap,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    match headers.get("Authorization") {
        Some(auth_header) => match auth_header.to_str() {
            Ok(auth_header_string) => {
                if let Some(player_profile) = player_sessions
                    .get_session(auth_header_string)
                    .await
                    .upgrade()
                    .map(|x| x.player_profile.clone())
                {
                    let chat_subscription = chat_service.get_receiving_handle();

                    // This stream takes in the needed state and automatically times out and retries in case a session has expired
                    let stream =
                        stream::unfold((chat_subscription, player_profile), async |mut state| {
                            match state.1.session.extend_session() {
                                true => {
                                    match timeout(
                                        Duration::from_secs(MESSAGE_REDRIVE_TIMEOUT_SECONDS),
                                        async { state.0.recv().await },
                                    )
                                    .await
                                    {
                                        Ok(received_message) => match received_message {
                                            Ok(chat_message) => {
                                                match Event::default().json_data(chat_message) {
                                                    Ok(serialized_event) => {
                                                        Some((Ok(serialized_event), state))
                                                    }
                                                    Err(e) => {
                                                        warn!(
                                                            "Error serializing chat message: {:?}",
                                                            e
                                                        );
                                                        None
                                                    }
                                                }
                                            }
                                            // We could probably do something here for lagged messages but just drop the connection for now
                                            Err(_) => None,
                                        },
                                        Err(_timed_out) => {
                                            // Timed out, so we just want to re-process and revalidate the session timeout
                                            Some((Ok(Event::default()), state))
                                        }
                                    }
                                }
                                false => None,
                            }
                        });

                    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
                } else {
                    Err(StatusCode::UNAUTHORIZED)
                }
            }
            Err(_) => Err(StatusCode::UNAUTHORIZED),
        },
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

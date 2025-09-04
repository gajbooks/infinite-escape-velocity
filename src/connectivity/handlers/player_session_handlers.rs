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

use axum::http::HeaderMap;
use axum::http::header::AUTHORIZATION;
use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use ts_rs::TS;

use crate::connectivity::player_info::player_profile::AuthType;

use crate::connectivity::player_info::{
    player_profiles::PlayerProfiles, player_sessions::PlayerSessions,
};
use crate::connectivity::services::ecs_communication_service::EcsCommunicationService;

#[derive(Serialize, TS)]
#[ts(export, export_to = "players/")]
pub struct LoginPlayerResponse {
    session_token: String,
}

pub async fn login_player(
    State((player_profiles, player_sessions, spawn_service)): State<(
        PlayerProfiles,
        PlayerSessions,
        EcsCommunicationService,
    )>,
    Json(request): Json<AuthType>,
) -> Result<Json<LoginPlayerResponse>, StatusCode> {
    match player_profiles.validate_login_request(&request).await {
        Ok(valid_profile) => {
            let session_token = player_sessions
                .create_session(valid_profile, &spawn_service)
                .await;
            Ok(LoginPlayerResponse { session_token }.into())
        }
        Err(()) => Err(StatusCode::UNAUTHORIZED),
    }
}

pub async fn validate_login(
    State(player_sessions): State<PlayerSessions>,
    headers: HeaderMap,
) -> StatusCode {
    match headers.get(AUTHORIZATION) {
        Some(auth_header) => match auth_header.to_str() {
            Ok(auth_header_string) => {
                if let Some(true) = player_sessions
                    .get_session(auth_header_string)
                    .await
                    .upgrade()
                    .map(|x| x.player_profile.session.extend_session())
                {
                    StatusCode::NO_CONTENT
                } else {
                    StatusCode::UNAUTHORIZED
                }
            }
            Err(_) => StatusCode::UNAUTHORIZED,
        },
        None => StatusCode::UNAUTHORIZED,
    }
}

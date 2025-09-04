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

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::connectivity::player_info::player_profiles::PlayerProfiles;

#[derive(Serialize, TS)]
#[ts(export, export_to = "players/")]
pub struct EphemeralPlayerResponse {
    id: String
}

pub async fn create_new_ephemeral_player(
    State(state): State<PlayerProfiles>,
) -> Result<Json<EphemeralPlayerResponse>, StatusCode> {
    match state.create_player_with_token().await {
        Ok(created) => {
            Ok(EphemeralPlayerResponse{id: created}.into())
        },
        Err(()) => Err(StatusCode::CONFLICT),
    }
}

#[derive(Deserialize, TS)]
#[ts(export, export_to = "players/")]
pub struct CreateUsernamePlayerRequest {
    username: String,
    password: String
}

pub async fn create_new_username_player(
    State(state): State<PlayerProfiles>,
    Json(request): Json<CreateUsernamePlayerRequest>
) -> StatusCode {
    match state.create_player_with_username_and_password(&request.username, &request.password).await {
        Ok(_username) => {
            StatusCode::OK
        },
        Err(()) => StatusCode::CONFLICT,
    }
}

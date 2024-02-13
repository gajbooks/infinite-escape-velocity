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

use std::sync::Arc;

use axum::{extract::{Path, State}, http::{header, StatusCode}, response::{IntoResponse, Response}};

use crate::backend::configuration_file_loaders::asset_file_cache::AssetFileCache;

#[derive(Clone)]
pub struct AssetServerState {
    pub assets: Arc<AssetFileCache>
}

pub async fn asset_handler(
    State(state): State<AssetServerState>,
    Path(asset_name): Path<String>
) -> Response {
    let assets = state.assets;
    match assets.get_asset_data(&asset_name) {
        Some((asset_info, data)) => {
            let guessed_extension = mime_guess::from_path(&asset_info.filename).first_or(mime::TEXT_PLAIN);
            (StatusCode::OK, [(header::CONTENT_TYPE, guessed_extension.essence_str())], data).into_response()
        },
        None => {
            (StatusCode::NOT_FOUND, format!("Asset not found: {}", asset_name)).into_response()
        },
    }
}
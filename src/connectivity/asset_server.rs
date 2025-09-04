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

use crate::{backend::configuration_file_loaders::asset_file_cache::AssetFileCache, configuration_file_structures::asset_definition_file::AssetResources};

#[derive(Clone)]
pub struct AssetServerState {
    pub assets: Arc<AssetFileCache>
}

pub async fn asset_by_name(
    State(state): State<AssetServerState>,
    Path(asset_name): Path<String>
) -> Response {
    let assets = state.assets;
    match assets.get_asset_data_by_name(&asset_name) {
        Some((asset_info, data)) => {
            match asset_info.asset_type {
                AssetResources::Meta(meta) => {
                    (StatusCode::OK, [(header::CONTENT_TYPE, mime::APPLICATION_JSON.essence_str())], axum::Json(meta)).into_response()
                },
                _ => {
                    match asset_info.asset_type.try_get_filename() {
                        Some(filename) => {
                            let guessed_extension = mime_guess::from_path(filename).first_or(mime::TEXT_PLAIN);
                            match data {
                                Some(data) => {
                                    (StatusCode::OK, [(header::CONTENT_TYPE, guessed_extension.essence_str())], data).into_response()
                                },
                                None => {
                                    (StatusCode::INTERNAL_SERVER_ERROR, "Non-meta asset does not have associated file data").into_response()
                                },
                            }
                        },
                        None => {
                            (StatusCode::INTERNAL_SERVER_ERROR, "Non-meta asset does not have an associated filename for MIME determination").into_response()
                        },
                    }
                }
            }
        },
        None => {
            (StatusCode::NOT_FOUND, format!("Asset not found: {}", asset_name)).into_response()
        },
    }
}

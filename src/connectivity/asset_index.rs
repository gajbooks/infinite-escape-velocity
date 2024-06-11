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

use std::{collections::HashMap, sync::Arc};

use axum::{extract::State, Json};

use crate::configuration_file_structures::reference_types::{AssetIndexReference, AssetReference};

use super::asset_index_response::{AssetIndexResponse, AssetIndexValue};

pub struct AssetIndex {
    assets: Vec<AssetReference>,
    asset_reverse_lookup: HashMap<AssetReference, AssetIndexReference>,
}

impl AssetIndex {
    pub fn new(assets: impl Iterator<Item = AssetReference>) -> Self {
        let mut asset_reverse_lookup = HashMap::<AssetReference, AssetIndexReference>::new();
        let assets = std::iter::once("default_asset".to_string())
            .chain(assets)
            .enumerate()
            .map(|(id, name)| {
                asset_reverse_lookup.insert(name.clone(), id as u64);
                tracing::trace!("Added asset {} to asset index with id {}", &name, id);
                name
            })
            .collect();
        Self {
            assets: assets,
            asset_reverse_lookup: asset_reverse_lookup,
        }
    }

    pub fn lookup_asset_by_name(&self, name: &str) -> Option<&AssetIndexReference> {
        self.asset_reverse_lookup.get(name)
    }

    pub fn lookup_asset_by_id(&self, id: AssetIndexReference) -> Option<&String> {
        self.assets.get(id as usize)
    }
}

#[derive(Clone)]
pub struct AssetIndexState {
    pub assets: Arc<AssetIndex>,
}

pub async fn get_asset_index(State(state): State<AssetIndexState>) -> Json<AssetIndexResponse> {
    axum::Json(AssetIndexResponse {
        asset_index_list: state
            .assets
            .assets
            .iter()
            .enumerate()
            .map(|(id, name)| AssetIndexValue {
                id: id as u64,
                name: name.clone(),
            })
            .collect(),
    })
}

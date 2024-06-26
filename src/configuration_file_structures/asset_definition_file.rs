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

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::reference_types::AssetReference;

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[ts(export, export_to = "assets/")]
pub enum GraphicsType {
    SimpleSquareRotationalSpriteSheet {
        sprite_count_x: u32,
        sprite_count_y: u32,
        image_data_asset: AssetReference,
    },
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[ts(export, export_to = "assets/")]
pub enum MetaAsset {
    Graphics(GraphicsType),
}

#[derive(Clone, Deserialize, Debug, Serialize, TS, Eq, PartialEq)]
#[ts(export, export_to = "assets/")]
pub enum AssetType {
    Image,
    Sound,
    Text,
    Meta,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[ts(export, export_to = "assets/")]
pub enum AssetResources {
    Image(String),
    Sound(String),
    Text(String),
    Meta(MetaAsset),
}

impl AssetResources {
    pub fn get_asset_type_from_resource(&self) -> AssetType {
        match self {
            AssetResources::Image(_) => AssetType::Image,
            AssetResources::Sound(_) => AssetType::Sound,
            AssetResources::Text(_) => AssetType::Text,
            AssetResources::Meta(_) => AssetType::Meta,
        }
    }
}

impl AssetResources {
    pub fn try_get_filename(&self) -> Option<&str> {
        match self {
            AssetResources::Image(filename) => Some(filename),
            AssetResources::Sound(filename) => Some(filename),
            AssetResources::Text(filename) => Some(filename),
            AssetResources::Meta(_metadata) => None,
        }
    }
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[ts(export, export_to = "assets/")]
pub struct AssetDefinition {
    pub asset_name: AssetReference,
    pub asset_type: AssetResources,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[ts(export, export_to = "assets/")]
pub struct AssetDefinitionFile {
    pub assets: Vec<AssetDefinition>,
}

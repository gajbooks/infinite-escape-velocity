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

#[derive(Clone, Copy, Deserialize, Debug, Serialize, TS)]
#[ts(export, export_to = "webapp/bindings/assets/")]
pub enum GraphicsType {
    StaticImage,
    SimpleSquareRotationalSpriteSheet {
        sprite_count_x: u32,
        sprite_count_y: u32
    }
}

#[derive(Clone, Copy, Deserialize, Debug, Serialize, TS)]
#[ts(export, export_to = "webapp/bindings/assets/")]
pub enum AssetType {
    Graphics(GraphicsType),
    Sound,
    Music,
    Text
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[ts(export, export_to = "webapp/bindings/assets/")]
pub struct AssetDefinitionFile {
    pub asset_name: String,
    pub asset_type: AssetType,
    pub filename: String
}
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

use serde::Serialize;
use ts_rs::TS;

#[derive(Clone, Serialize, Debug, TS)]
pub struct AssetIndexValue {
    pub id: u64,
    pub name: String,
}

#[derive(Serialize, Debug, TS)]
#[ts(export)]
pub struct AssetIndexResponse {
    pub asset_index_list: Vec<AssetIndexValue>,
}

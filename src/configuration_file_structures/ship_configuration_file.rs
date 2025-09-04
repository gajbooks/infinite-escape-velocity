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

use serde::Deserialize;

use crate::{
    backend::configuration_file_loaders::definition_caches::list_required_assets::ListRequiredAssets,
    configuration_file_structures::{
        asset_definition_file::AssetType,
        reference_types::{AssetReference, ShipReference},
    },
};

#[derive(Deserialize)]
pub struct ShipRecord {
    pub ship_reference: ShipReference,
    pub ship_display_name: AssetReference,
    pub combat: ShipRecordCombatProperties,
    pub display: ShipRecordDisplayProperties,
    pub motion: ShipRecordMotionProperties,
}

impl ListRequiredAssets for ShipRecord {
    fn get_required_asset_list(&self) -> Vec<(&AssetReference, AssetType)> {
        let mut required_assets = vec![(&self.ship_display_name, AssetType::Text)];
        required_assets.extend(self.display.get_required_asset_list());

        required_assets
    }
}

#[derive(Deserialize)]
pub struct ShipRecordDisplayProperties {
    pub display_asset: AssetReference,
    pub display_radius: f32,
}

impl ListRequiredAssets for ShipRecordDisplayProperties {
    fn get_required_asset_list(&self) -> Vec<(&AssetReference, AssetType)> {
        vec![(&self.display_asset, AssetType::Meta)]
    }
}

#[derive(Deserialize)]
pub struct ShipRecordMotionProperties {
    base_acceleration: f32,
    base_maximum_speed: f32,
    base_yaw_rate: f32,
}

#[derive(Deserialize)]
pub struct ShipRecordCombatProperties {
    base_hull: f32,
    base_shield: f32,
    base_shield_regen_rate: f32,
}

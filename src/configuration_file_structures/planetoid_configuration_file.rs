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

use crate::backend::configuration_file_loaders::definition_caches::list_required_assets::ListRequiredAssets;

use super::{
    asset_definition_file::AssetType,
    reference_types::{AssetReference, PlanetoidReference},
};

#[derive(Deserialize)]
pub struct PlanetoidMayBeLandedOn {
    pub landing_radius: f32,
    pub backdrop_image_asset: AssetReference,
    pub text_description_asset: AssetReference,
    pub features: Option<PlanetoidFeatures>,
    pub opinion: Option<String>, // Eventually will describe the "politics" of a planet and will decide if you are allowed to land, default to yes
}

impl ListRequiredAssets for PlanetoidMayBeLandedOn {
    fn get_required_asset_list(&self) -> Vec<(&AssetReference, AssetType)> {
        vec![
            (&self.backdrop_image_asset, AssetType::Image),
            (&self.text_description_asset, AssetType::Text),
        ]
    }
}

#[derive(Deserialize)]
pub struct PlanetoidFeatures {
    // Eventually should be string identifiers for ship features, like pre-defined shipyards, outfitters, bar, BBS, etc which can be modified by quests or other events and are potentially reusable
}

#[derive(Deserialize)]
pub struct PlanetoidRecord {
    pub planetoid_reference: PlanetoidReference,
    pub planetoid_display_name: String,
    pub display_asset: AssetReference,
    pub display_radius: f32,
    pub x: f64,
    pub y: f64,
    pub may_be_landed_on: Option<PlanetoidMayBeLandedOn>,
}

impl ListRequiredAssets for PlanetoidRecord {
    fn get_required_asset_list(&self) -> Vec<(&AssetReference, AssetType)> {
        let mut required_assets = vec![(&self.display_asset, AssetType::Image)];

        match &self.may_be_landed_on {
            Some(has) => {
                required_assets.extend(has.get_required_asset_list());
            }
            None => (),
        }

        required_assets
    }
}

#[derive(Deserialize)]
pub struct PlanetoidConfigurationFile {
    pub definitions: Vec<PlanetoidRecord>,
}

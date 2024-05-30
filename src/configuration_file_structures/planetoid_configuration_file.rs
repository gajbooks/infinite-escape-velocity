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

use super::reference_string_types::AssetReference;

#[derive(Deserialize)]
pub struct PlanetoidMayBeLandedOn {
    pub landing_radius: f32,
    pub backdrop_image_asset: AssetReference,
    pub text_description_asset: AssetReference,
    pub features: Option<PlanetoidFeatures>,
    pub opinion: Option<String> // Eventually will describe the "politics" of a planet and will decide if you are allowed to land, default to yes
}

#[derive(Deserialize)]
pub struct PlanetoidFeatures {
    // Eventually should be string identifiers for ship features, like pre-defined shipyards, outfitters, bar, BBS, etc which can be modified by quests or other events and are potentially reusable
}

#[derive(Deserialize)]
pub struct PlanetoidRecord {
    pub planetoid_name: String,
    pub display_asset: AssetReference,
    pub display_radius: f32,
    pub x: f64,
    pub y: f64,
    pub may_be_landed_on: Option<PlanetoidMayBeLandedOn>
}

#[derive(Deserialize)]
pub struct PlanetoidConfigurationFile {
    pub definitions: Vec<PlanetoidRecord>,
}

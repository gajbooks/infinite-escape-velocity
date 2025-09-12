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

use crate::configuration_file_structures::reference_types::{AssetIndexReference, ObjectId};

use super::view_layers::ViewLayers;

#[derive(Serialize, Debug, TS)]
pub struct VelocityMessage {
    pub vx: f32,
    pub vy: f32,
}

#[derive(Serialize, Debug, TS)]
pub struct RotationMessage {
    pub rotation: f32,
}

#[derive(Serialize, Debug, TS)]
pub struct AngularVelocityMessage {
    pub angular_velocity: f32,
}

#[derive(Serialize, Debug, TS)]
#[ts(export)]
pub struct DynamicObjectUpdateData {
    pub x: f64,
    pub y: f64,
    pub rotation: Option<RotationMessage>,
    pub velocity: Option<VelocityMessage>,
    pub angular_velocity: Option<AngularVelocityMessage>,
    pub id: ObjectId,
}

#[derive(Serialize, Debug, TS)]
#[ts(export)]
pub struct DynamicObjectCreationData {
    pub id: ObjectId,
    pub object_asset: AssetIndexReference,
    pub view_layer: ViewLayers,
    pub display_radius: f32,
}

#[derive(Serialize, Debug, TS)]
#[ts(export)]
pub struct DynamicObjectDestructionData {
    pub id: ObjectId,
}

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

use crate::shared_types::*;

#[derive(Serialize, Debug, TS)]
#[ts(export, export_to="webapp/bindings/")]
pub struct DynamicObjectMessageData {
    pub x: f64,
    pub y: f64,
    pub rotation: f32,
    pub vx: f32,
    pub vy: f32,
    pub angular_velocity: f32,
    pub object_type: ObjectType,
    pub id: IdType
}

#[derive(Serialize, Debug, TS)]
#[ts(export, export_to="webapp/bindings/")]
pub struct DynamicObjectCreationData {
    pub id: IdType
}

#[derive(Serialize, Debug, TS)]
#[ts(export, export_to="webapp/bindings/")]
pub struct DynamicObjectDestructionData {
    pub id: IdType
}
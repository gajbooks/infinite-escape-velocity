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

use bevy_ecs::entity::Entity;
use serde::Serialize;
use ts_rs::TS;

#[derive(Serialize, Debug, TS)]
pub struct ExternalEntity {
    pub generation: u32,
    pub index: u32
}

impl From<Entity> for ExternalEntity {
    fn from(value: Entity) -> Self {
        ExternalEntity{generation: value.generation(), index: value.index()}
    }
}

#[derive(Serialize, Debug, TS)]
#[ts(export, export_to="webapp/bindings/")]
pub struct DynamicObjectMessageData {
    pub x: f64,
    pub y: f64,
    pub rotation: f32,
    pub vx: f32,
    pub vy: f32,
    pub angular_velocity: f32,
    pub object_type: String,
    pub id: ExternalEntity
}

#[derive(Serialize, Debug, TS)]
#[ts(export, export_to="webapp/bindings/")]
pub struct DynamicObjectCreationData {
    pub id: ExternalEntity
}

#[derive(Serialize, Debug, TS)]
#[ts(export, export_to="webapp/bindings/")]
pub struct DynamicObjectDestructionData {
    pub id: ExternalEntity
}
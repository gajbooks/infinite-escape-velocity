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

use super::graphics_configuration::*;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct MovementParameters {
    pub maximum_speed: f32,
    pub maximum_acceleration: f32,
    pub maximum_angular_velocity: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CirleParameters {
    pub radius: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RoundedTubeParameters {
    pub radius: f32,
    pub length: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CollisionParameters {
    pub circle: Option<CirleParameters>,
    pub rounded_tube: Option<RoundedTubeParameters>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PositionParameters {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LifeParameters {
    pub structure_points: f32,
    pub shield_points: f32,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct ObjectTypeParameters {
    pub author: String,
    pub object_type: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "variant")]
pub enum ObjectVariant {
    Ship {
        movement_parameters: MovementParameters,
        collision_parameters: CollisionParameters,
        graphics_parameters: GraphicsParameters,
        life_parameters: LifeParameters,
    },
    Munition {
        damage: f32,
        movement_parameters: MovementParameters,
        collision_parameters: CollisionParameters,
        graphics_parameters: GraphicsParameters,
        life_parameters: LifeParameters,
    },
    Planet {
        position_parameters: PositionParameters,
        collision_parameters: CollisionParameters,
        graphics_parameters: GraphicsParameters,
    },
}

#[derive(Clone, Debug, Deserialize)]
pub struct ObjectConfigurationRecord {
    pub object_type: ObjectTypeParameters,
    pub object: ObjectVariant,
}

impl ObjectVariant {
    pub fn get_graphics_parameters(&self) -> Option<&GraphicsParameters> {
        match self {
            ObjectVariant::Ship {
                graphics_parameters,
                ..
            } => Some(graphics_parameters),
            ObjectVariant::Munition {
                graphics_parameters,
                ..
            } => Some(graphics_parameters),
            ObjectVariant::Planet {
                graphics_parameters,
                ..
            } => Some(graphics_parameters),
        }
    }
}

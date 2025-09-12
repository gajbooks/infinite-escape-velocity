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

use bevy_ecs::{component::Component, entity::Entity};

use crate::{backend::spatial_optimizer::hash_cell_size::HashCellSize, shared_types::Health};

#[derive(Component)]
pub struct DamagingEntityComponent {
    pub alliegence: Entity,
    pub hull_damage: Health,
    pub shield_damage: Health,
}

pub struct DamagingEntityCollisionMarker;

impl HashCellSize for DamagingEntityCollisionMarker {}

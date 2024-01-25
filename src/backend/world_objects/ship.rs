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

use bevy_ecs::bundle::Bundle;

use crate::{backend::shape::{PointData, Shape}, shared_types::Coordinates};

use super::{
    components::{collision_component::CollisionMarker, position_component::PositionComponent}, server_viewport::Displayable,
};

#[derive(Bundle)]
pub struct ShipBundle {
    pub displayable: Displayable,
    pub displayable_collision_marker: CollisionMarker<Displayable>,
    pub position: PositionComponent
}

impl ShipBundle {
    pub fn new(name: &str, position: Coordinates) -> ShipBundle {
        ShipBundle {
            displayable: Displayable{object_type: format!("Ship {}", name)},
            displayable_collision_marker: CollisionMarker::<Displayable>::new(Shape::Point(PointData{point: position})),
            position: PositionComponent{position: position}
        }
    }
}

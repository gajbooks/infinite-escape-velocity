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

use bevy_ecs::system::Query;

use crate::backend::world_objects::components::{collision_component::{CollisionEvaluatorComponent, CollisionSourceComponent}, position_component::PositionComponent};

pub fn update_collisions_with_position<T: Send + Sync>(mut sender: Query<(&mut CollisionEvaluatorComponent<T>, &PositionComponent)>, mut receiver: Query<(&mut CollisionSourceComponent<T>, &PositionComponent)>) {
    sender.par_iter_mut().for_each(|(mut collider, position)| {
        collider.shape = collider.shape.move_center(position.position);
    });
    receiver.par_iter_mut().for_each(|(mut collider, position)| {
        collider.shape = collider.shape.move_center(position.position);
    });
}

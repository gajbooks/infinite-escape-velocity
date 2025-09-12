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

use crate::backend::{
    spatial_optimizer::hash_cell_size::HashCellSize,
    world_objects::components::{
        collision_component::{CollisionEvaluatorComponent, CollisionSourceComponent},
        rotation_component::RotationComponent,
    },
};

pub fn update_collisions_with_rotation<T: ?Sized + HashCellSize>(
    mut sender: Query<(&mut CollisionEvaluatorComponent<T>, &RotationComponent)>,
    mut receiver: Query<(&mut CollisionSourceComponent<T>, &RotationComponent)>,
) {
    sender.par_iter_mut().for_each(|(mut collider, rotation)| {
        collider.shape = collider.shape.set_rotation(rotation.rotation);
    });
    receiver
        .par_iter_mut()
        .for_each(|(mut collider, rotation)| {
            collider.shape = collider.shape.set_rotation(rotation.rotation);
        });
}

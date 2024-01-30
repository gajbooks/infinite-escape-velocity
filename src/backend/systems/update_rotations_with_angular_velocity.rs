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

use bevy_ecs::system::{Query, Res};

use crate::backend::{resources::delta_t_resource::DeltaTResource, world_objects::components::{angular_velocity_component::AngularVelocityComponent, rotation_component::RotationComponent}};

pub fn update_rotations_with_angular_velocity(mut movable: Query<(&mut RotationComponent, &AngularVelocityComponent)>, time: Res<DeltaTResource>) {
    let delta_t = time.last_tick.as_secs_f32();
    movable.par_iter_mut().for_each(|(mut rotation, angular_velocity)| {
        rotation.rotation = (rotation.rotation + (angular_velocity.angular_velocity * delta_t)).signed();
    });
}
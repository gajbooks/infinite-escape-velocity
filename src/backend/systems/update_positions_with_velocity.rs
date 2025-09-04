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

use crate::{backend::{resources::delta_t_resource::DeltaTResource, world_objects::components::{position_component::PositionComponent, velocity_component::VelocityComponent}}, shared_types::{GlobalCoordinateType, WorldCoordinates}};

pub fn update_positions_with_velocity(mut movable: Query<(&mut PositionComponent, &VelocityComponent)>, time: Res<DeltaTResource>) {
    let delta_t = time.get_last_tick_duration().as_secs_f32();
    movable.par_iter_mut().for_each(|(mut position, velocity)| {
        position.position = position.position + (velocity.velocity * delta_t).cast::<GlobalCoordinateType>().cast_unit::<WorldCoordinates>();
    });
}
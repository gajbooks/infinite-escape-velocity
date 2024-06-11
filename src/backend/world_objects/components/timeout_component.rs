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

use std::time::Duration;

use bevy_ecs::{
    component::Component,
    entity::Entity,
    system::{ ParallelCommands, Query, Res},
};

use crate::backend::resources::delta_t_resource::DeltaTResource;

#[derive(Component)]
pub struct TimeoutComponent {
    pub spawn_time: Duration,
    pub lifetime: Duration,
}

pub fn check_despawn_times(
    timeouts: Query<(Entity, &TimeoutComponent)>,
    time: Res<DeltaTResource>,
    commands: ParallelCommands,
) {
    timeouts.par_iter().for_each(|(entity, timeout)| {
        if time.total_time - timeout.spawn_time > timeout.lifetime {
            commands.command_scope(|mut commands| {
                commands.entity(entity).despawn();
            })
        }
    });
}

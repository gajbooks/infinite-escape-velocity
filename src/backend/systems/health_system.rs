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

use bevy_ecs::{entity::Entity, system::{ParallelCommands, Query, Res}};

use crate::backend::{resources::delta_t_resource::DeltaTResource, world_objects::components::{health_properties_component::HealthPropertiesComponent, health_state_component::HealthStateComponent}};

pub fn evaluate_health(delta_t: Res<DeltaTResource>, mut healthy_entities: Query<(Entity, &mut HealthStateComponent, &HealthPropertiesComponent)>, commands: ParallelCommands) {
    healthy_entities.par_iter_mut().for_each(|(entity, mut health_state, health_properties)| {
        if health_state.hull < 0.0 {
            commands.command_scope(|mut comm| comm.entity(entity).despawn());
            return;
        }
        
        let delta_t = delta_t.get_last_tick_duration();
        
        let hull_regeneration = health_properties.hull_regeneration_rate * delta_t;
        let shield_regeneration = health_properties.shield_regeneration_rate * delta_t;

        health_state.hull = health_properties.maximum_hull.min(health_state.hull + hull_regeneration);
        health_state.shield = health_properties.maximum_shield.min(health_state.shield + shield_regeneration);
    });
}
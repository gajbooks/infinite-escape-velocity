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

use bevy_ecs::{
    entity::Entity,
    system::{ParallelCommands, Query},
};

use crate::backend::world_objects::components::{
    collision_component::CollisionEvaluatorComponent,
    damaging_entity_component::{DamagingEntityCollisionMarker, DamagingEntityComponent},
    health_state_component::HealthStateComponent,
};

pub fn evaluate_damage(
    damage_dealing_entities: Query<(
        Entity,
        &DamagingEntityComponent,
        &CollisionEvaluatorComponent<DamagingEntityCollisionMarker>,
    )>,
    mut damage_receiving_entities: Query<&mut HealthStateComponent>,
    commands: ParallelCommands,
) {
    for (damaging_entity, damaging_component, collided_entities) in damage_dealing_entities.iter() {
        'damage_entity_targets: for target_entity in collided_entities.list.iter() {
            // Don't target the launcher
            if *target_entity == damaging_component.alliegence {
                continue;
            }

            match damage_receiving_entities.get_mut(*target_entity) {
                Ok(mut target_health) => {
                    let mut remaining_shield_health =
                        target_health.shield - damaging_component.shield_damage;

                    if remaining_shield_health <= 0.0 {
                        let overkill_shield_damage = remaining_shield_health.abs();
                        remaining_shield_health = 0.0;

                        let overkill_damage_proportion =
                            overkill_shield_damage / damaging_component.shield_damage;

                        let hull_damage_dealt =
                            overkill_damage_proportion * damaging_component.hull_damage;

                        target_health.hull -= hull_damage_dealt;
                    }

                    target_health.shield = remaining_shield_health;

                    commands.command_scope(|mut comm| comm.entity(damaging_entity).despawn());
                    break 'damage_entity_targets;
                }
                Err(_) => (),
            }
        }
    }
}

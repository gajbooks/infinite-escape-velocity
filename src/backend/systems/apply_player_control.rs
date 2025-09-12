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
    component::{Component, Mutable},
    entity::Entity,
    hierarchy::ChildOf,
    system::Query,
};
use euclid::num::Zero;

use crate::{
    backend::{
        components::session::player_session_component::PlayerSessionComponent,
        world_objects::components::{
            angular_velocity_properties_component::AngularVelocityPropertiesComponent, angular_velocity_state_component::AngularVelocityStateComponent, maximum_acceleration_properties_component::MaximumAccelerationPropertiesComponent, player_controlled_component::PlayerControlledComponent
        },
    },
    shared_types::{AccelerationScalar, AngularVelocity},
};

pub trait PlayerControllablePhysics {
    fn set_acceleration(&mut self, acceleration: AccelerationScalar);
}

pub fn apply_player_control<T: PlayerControllablePhysics + Component<Mutability = Mutable>>(
    mut controllable: Query<(Entity, &PlayerControlledComponent, &mut T, &MaximumAccelerationPropertiesComponent, &ChildOf)>,
    mut angular_velocity_components: Query<(
        &mut AngularVelocityStateComponent,
        &AngularVelocityPropertiesComponent,
    )>,
    sessions: Query<&PlayerSessionComponent>,
) {
    controllable.iter_mut().for_each(
        |(entity, _player_controls, mut physics_component, acceleration_properties, parent_session)| {
            let session = match sessions.get(parent_session.parent()) {
                Ok(session) => session,
                Err(_) => return,
            };

            let input_status = session.input_status;

            if input_status.forward {
                physics_component
                    .set_acceleration(acceleration_properties.maximum_acceleration);
            } else {
                physics_component.set_acceleration(AccelerationScalar::zero());
            }

            match angular_velocity_components.get_mut(entity) {
                Ok((mut angular_velocity, angular_velocity_properties)) => {
                    if input_status.left && !input_status.right {
                        angular_velocity.angular_velocity =
                            -angular_velocity_properties.maximum_angular_velocity;
                    } else if input_status.right && !input_status.left {
                        angular_velocity.angular_velocity =
                            angular_velocity_properties.maximum_angular_velocity;
                    } else {
                        angular_velocity.angular_velocity = -AngularVelocity::zero();
                    }
                }
                Err(_) => (), // We can't change angular velocity directly on this entity
            };
        },
    );
}

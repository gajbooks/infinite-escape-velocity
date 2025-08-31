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
            angular_velocity_component::AngularVelocityComponent,
            player_controlled_component::PlayerControlledComponent,
        },
    },
    shared_types::{AccelerationScalar, AngularVelocity},
};

pub trait PlayerControllablePhysics {
    fn set_acceleration(&mut self, acceleration: AccelerationScalar);
}

const PLAYER_ACCELERATION_PLACEHOLDER: f32 = 40.0;
const PLAYER_ANGULAR_VELOCITY_PLACEHOLDER: f32 = std::f32::consts::PI / 2.0;

pub fn apply_player_control<T: PlayerControllablePhysics + Component<Mutability = Mutable>>(
    mut controllable: Query<(Entity, &PlayerControlledComponent, &mut T, &ChildOf)>,
    mut angular_velocity_components: Query<&mut AngularVelocityComponent>,
    sessions: Query<&PlayerSessionComponent>,
) {
    controllable.iter_mut().for_each(
        |(entity, _player_controls, mut physics_component, parent_session)| {
            let session = match sessions.get(parent_session.parent()) {
                Ok(session) => session,
                Err(_) => return,
            };

            let player_session = match session.session.upgrade() {
                Some(valid) => valid,
                None => return,
            };

            let input_status = player_session.input_status.blocking_read();

            if input_status.forward {
                physics_component
                    .set_acceleration(AccelerationScalar::new(PLAYER_ACCELERATION_PLACEHOLDER));
            } else {
                physics_component.set_acceleration(AccelerationScalar::zero());
            }

            match angular_velocity_components.get_mut(entity) {
                Ok(mut angular_velocity) => {
                    if input_status.left && !input_status.right {
                        angular_velocity.angular_velocity =
                            -AngularVelocity::radians(PLAYER_ANGULAR_VELOCITY_PLACEHOLDER);
                    } else if input_status.right && !input_status.left {
                        angular_velocity.angular_velocity =
                            AngularVelocity::radians(PLAYER_ANGULAR_VELOCITY_PLACEHOLDER);
                    } else {
                        angular_velocity.angular_velocity = -AngularVelocity::zero();
                    }
                }
                Err(_) => (), // We can't change angular velocity directly on this entity
            };
        },
    );
}

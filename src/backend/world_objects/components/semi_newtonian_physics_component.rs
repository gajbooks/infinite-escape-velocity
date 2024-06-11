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

use bevy_ecs::component::Component;

use crate::{backend::systems::apply_player_control::PlayerControllablePhysics, shared_types::{ AccelerationScalar, Speed}};

#[derive(Component)]
pub struct SemiNewtonianPhysicsComponent {
    pub maximum_speed: Speed,
    pub thrust: AccelerationScalar
}

impl SemiNewtonianPhysicsComponent {
    pub fn new(
        maximum_speed: Speed,
    ) -> SemiNewtonianPhysicsComponent {
        SemiNewtonianPhysicsComponent {
            maximum_speed,
            thrust: AccelerationScalar::default()
        }
    }
}

impl PlayerControllablePhysics for SemiNewtonianPhysicsComponent {
    fn set_acceleration(&mut self, acceleration: AccelerationScalar) {
        self.thrust = acceleration;
    }
}

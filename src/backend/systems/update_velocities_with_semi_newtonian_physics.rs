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

const EXCESS_REDUCTION_EXPONENT: f32 = 2.0;
const EXCESS_REDUCTION_CONSTANT: f32 = 1.0;

use std::f32;

use bevy_ecs::system::{Query, Res};
use euclid::num::Zero;

use crate::{
    backend::{
        resources::delta_t_resource::DeltaTResource,
        world_objects::components::{
            angular_velocity_component::AngularVelocityComponent,
            rotation_component::RotationComponent,
            semi_newtonian_physics_component::SemiNewtonianPhysicsComponent,
            velocity_component::VelocityComponent,
        },
    },
    shared_types::{ Velocity, VelocityCoordinates},
};

pub fn update_velocities_with_semi_newtonian_physics(
    mut compatible_entities: Query<(
        &SemiNewtonianPhysicsComponent,
        &mut VelocityComponent,
        &RotationComponent,
        &AngularVelocityComponent,
    )>,
    delta_t: Res<DeltaTResource>,
) {
    let delta_t = delta_t.last_tick.as_secs_f32();
    compatible_entities.par_iter_mut().for_each(
        |(semi_newtonian_physics, mut velocity, rotation, angular_velocity)| {
            // Calculate the acceleration vector before rotation this tick (subtract to find previous tick angle)
            let thrust_vector_before = Velocity::from_angle_and_length(
                rotation.rotation - (angular_velocity.angular_velocity * delta_t),
                semi_newtonian_physics.thrust.get(),
            );

            // Calculate the acceleration vector after rotation this tick (implicit)
            let thrust_vector_after = Velocity::from_angle_and_length(
                rotation.rotation,
                semi_newtonian_physics.thrust.get(),
            );

            // Interpolate before and after values to find the correct direction of thrust
            let vector_thrust = thrust_vector_after.lerp(thrust_vector_before, 0.5);

            // Acceleration over a discrete time step delta_t
            let delta_t_acceleration = vector_thrust * delta_t;

            // New, unclamped velocity to allow for turning
            let new_velocity =
                velocity.velocity + delta_t_acceleration.cast_unit::<VelocityCoordinates>();

            // Amount of excess velocity the new velocity value has beyond the maximum allowed
            let excess_speed = new_velocity.length() - semi_newtonian_physics.maximum_speed.get();

            // Only do anything if there is excess velocity
            let new_velocity = if excess_speed > f32::zero() {
                // Calculate how much excess speed we should trim from the new velocity
                // Trim the excess speed added by the most recent accelerations applied so acceleration can never exceed the maximum speed,
                // then decay the remaining excess speed by an exponential constant and a small constant value per delta_t
                let excess_falloff = ((excess_speed - delta_t_acceleration.length())
                    .min(f32::zero())
                    * (-delta_t / EXCESS_REDUCTION_EXPONENT).exp())
                    - (EXCESS_REDUCTION_CONSTANT * delta_t);

                // Create a vector velocity with the excess speed subtracted from it
                let velocity_less_excess_exponent =
                    new_velocity.with_max_length(new_velocity.length() - excess_falloff);

                // Ensure this correction never takes the speed below the maximum speed
                velocity_less_excess_exponent
                    .with_min_length(semi_newtonian_physics.maximum_speed.get())
            } else {
                new_velocity
            };

            // Update velocity component with limited speed
            velocity.velocity = new_velocity;
        },
    );
}

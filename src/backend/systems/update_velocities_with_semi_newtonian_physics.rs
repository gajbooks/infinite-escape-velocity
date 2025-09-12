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
const DRAG_CONSTANT: f32 = 1.0;
const MIN_DRAG_SPEED_PERCENT: f32 = 10.0;
const MAX_SPEED_EPSILON: f32 = 0.001;

use std::f32;

use bevy_ecs::system::{Query, Res};
use euclid::num::Zero;

use crate::{
    backend::{
        resources::delta_t_resource::DeltaTResource,
        world_objects::components::{
            angular_velocity_state_component::AngularVelocityStateComponent, maximum_speed_properties_component::MaximumSpeedPropertiesComponent, rotation_component::RotationComponent, semi_newtonian_physics_state_component::SemiNewtonianPhysicsStateComponent, velocity_component::VelocityComponent
        },
    },
    shared_types::{Acceleration, Velocity, VelocityCoordinates},
};

// This is an extrapolation of the velocity vector if we were under standard acceleration a for time t, with initial tangential velocity s
// Assuming you accelerate in a straight line continuously, s will eventually become insignificant with respect to a*t
// Resulting angle is given by r=atan2(s, a*t), inverse to find time t from angle r is t=(s*cot(r))/a,
// Then we add delta-time b to this calculated "duration of acceleration" for atan2(s, a*(b+(s*cot(r)/a)))
// This gives us the rotation angle we should have if we had realistically accelerated in an unconstrained world for this amount of time
// Unfortunately cotangent means we fight with infinities, so through the magic of WolframAlpha, we derive...
// atan2(s*sin(r), a*b*sin(r)+s*cos(r))
// Which is the formula used here, which covers [-PI, PI] and has no precision-based discontinuities within our usage range of [-PI/2, PI/2]
fn rotate_maximum_speed_vector(
    acceleration_vector: Acceleration,
    current_velocity_vector: Velocity,
    delta_t: f32,
) -> Velocity {
    let angle =
        current_velocity_vector.angle_to(acceleration_vector.cast_unit::<VelocityCoordinates>());
    let current_vel_magnitude = current_velocity_vector.length();
    let current_accel_magnitude = acceleration_vector.length();

    let (angle_sin, angle_cos) = angle.sin_cos();

    let current_vel_mag_times_sin = current_vel_magnitude * angle_sin;
    let current_vel_mag_times_cos = current_vel_magnitude * angle_cos;

    let accel_times_delta_t_times_sin = current_accel_magnitude * delta_t * angle_sin;

    let new_rotation_angle_relative_to_normal =
        current_vel_mag_times_sin.atan2(accel_times_delta_t_times_sin + current_vel_mag_times_cos);
    let rotation_adjustment = euclid::Angle {
        radians: new_rotation_angle_relative_to_normal,
    }
    .angle_to(angle);

    Velocity::from_angle_and_length(
        current_velocity_vector.angle_from_x_axis() + rotation_adjustment,
        current_vel_magnitude,
    )
}

pub fn update_velocities_with_semi_newtonian_physics(
    mut compatible_entities: Query<(
        &SemiNewtonianPhysicsStateComponent,
        &mut VelocityComponent,
        &RotationComponent,
        &AngularVelocityStateComponent,
        &MaximumSpeedPropertiesComponent,
    )>,
    delta_t: Res<DeltaTResource>,
) {
    let delta_t = delta_t.get_last_tick_duration();
    compatible_entities.par_iter_mut().for_each(
        |(semi_newtonian_physics, mut velocity, rotation, angular_velocity, maximum_speed_properties)| {
            // Calculate the acceleration vector before rotation this tick (subtract to find previous tick angle)
            let thrust_vector_before = Acceleration::from_angle_and_length(
                rotation.rotation - (angular_velocity.angular_velocity * delta_t),
                semi_newtonian_physics.thrust.get(),
            );

            // Calculate the acceleration vector after rotation this tick (implicit)
            let thrust_vector_after = Acceleration::from_angle_and_length(
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
            let excess_speed = new_velocity.length() - maximum_speed_properties.maximum_speed.get();

            // Only do anything if there is excess velocity above the margin and we are actually accelerating
            let new_velocity = if excess_speed > MAX_SPEED_EPSILON
                && vector_thrust.length() > MAX_SPEED_EPSILON
            {
                // Calculate how much excess speed we should trim from the new velocity
                // Trim the excess speed added by the most recent accelerations applied so acceleration can never exceed the maximum speed,
                // then decay the remaining excess speed by an exponential constant and a small constant value per delta_t
                let excess_falloff = (excess_speed * (-EXCESS_REDUCTION_EXPONENT * delta_t).exp())
                    - (EXCESS_REDUCTION_CONSTANT * delta_t).max(f32::zero());

                // Create a vector velocity with the excess speed subtracted from it
                let velocity_less_excess_exponent =
                    new_velocity.with_max_length(new_velocity.length() - excess_falloff);

                let rotated_velocity = rotate_maximum_speed_vector(
                    vector_thrust,
                    velocity_less_excess_exponent,
                    delta_t,
                );

                // Ensure this correction never takes the speed below the maximum speed
                rotated_velocity.with_min_length(maximum_speed_properties.maximum_speed.get())
            } else {
                new_velocity
            };

            // Apply general drag to velocity if we are below the minumum stopping speed
            let new_velocity = if new_velocity.length()
                <= (maximum_speed_properties.maximum_speed.get() / MIN_DRAG_SPEED_PERCENT)
            {
                new_velocity * (-DRAG_CONSTANT * delta_t).exp()
            } else {
                new_velocity
            };

            // Update velocity component with limited speed
            velocity.velocity = new_velocity;
        },
    );
}

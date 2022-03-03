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

use super::super::shared_types::*;
use super::motion_component::*;
use std::sync::*;
use std::sync::atomic::*;

pub trait ControllableObject {
    fn turn_left_for_tick(&self) {
        self.turn_left_for_tick_with_multiplier(1.0);
    }

    fn turn_right_for_tick(&self) {
        self.turn_right_for_tick_with_multiplier(1.0);
    }

    fn accelerate_forward_for_tick(&self) {
        self.accelerate_forward_for_tick_with_multiplier(1.0);
    }

    fn fire_for_tick(&self);
    fn is_firing(&self) -> bool;

    fn turn_left_for_tick_with_multiplier(&self, multiplier: f32);
    fn turn_right_for_tick_with_multiplier(&self, multiplier: f32);
    fn accelerate_forward_for_tick_with_multiplier(&self, multiplier: f32);

    fn stop_lateral_motion(&self);
    fn stop_rotational_motion(&self);

    fn tick(&self);
}

pub struct ControllableComponentShip {
    maximum_speed: LocalCoordinateType,
    maximum_acceleration: LocalCoordinateType,
    maximum_angular_velocity: LocalCoordinateType,
    motion_component: Arc<MaximumSpeedMotionComponent>,
    firing: AtomicBool
}

impl ControllableComponentShip {
    pub fn new(
        maximum_speed: LocalCoordinateType,
        maximum_acceleration: LocalCoordinateType,
        maximum_angular_velocity: LocalCoordinateType,
        motion_component: Arc<MaximumSpeedMotionComponent>) -> ControllableComponentShip
        {
            return ControllableComponentShip {
                maximum_speed: maximum_speed,
                maximum_acceleration: maximum_acceleration,
                maximum_angular_velocity: maximum_angular_velocity,
                motion_component: motion_component,
                firing: AtomicBool::from(false)};
        }
}

impl ControllableObject for ControllableComponentShip {
    fn turn_left_for_tick_with_multiplier(&self, multiplier: f32) {
        let multiplier = multiplier.clamp(0.0, 1.0);
        self.motion_component.set_velocity(None, None, Some(self.maximum_angular_velocity * multiplier * -1.0));
    }

    fn turn_right_for_tick_with_multiplier(&self, multiplier: f32) {
        let multiplier = multiplier.clamp(0.0, 1.0);
        self.motion_component.set_velocity(None, None, Some(self.maximum_angular_velocity * multiplier * 1.0));
    }

    fn accelerate_forward_for_tick_with_multiplier(&self, multiplier: f32) {
        let multiplier = multiplier.clamp(0.0, 1.0);
        self.motion_component.add_acceleration_along_pointing_direction(self.maximum_acceleration * multiplier);
    }

    fn fire_for_tick(&self) {
        self.firing.store(true, Ordering::Relaxed);
    }

    fn is_firing(&self) -> bool {
        return self.firing.load(Ordering::Relaxed);
    }

    fn stop_lateral_motion(&self) {
        self.motion_component.set_velocity(Some(0.0), Some(0.0), None);
    }

    fn stop_rotational_motion(&self) {
        self.motion_component.set_velocity(None, None, Some(0.0));
    }

    fn tick(&self) {
        self.motion_component.set_maximum_speed(self.maximum_speed);
        self.motion_component.set_maximum_acceleration(self.maximum_acceleration);
        self.stop_rotational_motion();
        self.firing.store(false, Ordering::Relaxed);
    }
}
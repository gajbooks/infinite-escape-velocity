use super::super::shared_types::*;
use super::motion_component::*;
use std::sync::*;

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
    motion_component: Arc<MaximumSpeedMotionComponent>
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
                motion_component: motion_component};
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
    }
}
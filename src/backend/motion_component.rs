use super::super::shared_types::*;
use std::sync::Mutex;

pub struct MotionComponent {
    coordinates: Mutex<CoordinatesVelocity>
}

impl MotionComponent {
    pub fn new() -> MotionComponent {
        return MotionComponent{coordinates: Mutex::new(CoordinatesVelocity{x: 0.0, y: 0.0, r: 0.0, dx: 0.0, dy: 0.0, dr: 0.0})};
    }

    pub fn new_from_position(position: &CoordinatesRotation) -> MotionComponent {
        return MotionComponent{coordinates: Mutex::new(CoordinatesVelocity{x: position.x, y: position.y, r: position.r, dx: 0.0, dy: 0.0, dr: 0.0})};
    }

    pub fn apply_velocity_tick(&self, delta_t: DeltaT) {
        let mut old = self.coordinates.lock().unwrap();
        *old = CoordinatesVelocity{x: old.x + (old.dx * delta_t) as f64, y: old.y + (old.dy * delta_t) as f64, r: old.r + (old.dr * delta_t), dx: old.dx, dy: old.dy, dr: old.dr};
    }

    pub fn get_coordinates(&self) -> CoordinatesVelocity {
        return self.coordinates.lock().unwrap().clone();
    }

    pub fn set_velocity(&self, dx: Option<LocalCoordinateType>, dy: Option<LocalCoordinateType>, dr: Option<LocalCoordinateType>) {
        let mut old = self.coordinates.lock().unwrap();
        *old = CoordinatesVelocity{x: old.x, y: old.y, r: old.r, dx: dx.unwrap_or(old.dx), dy: dy.unwrap_or(old.dy), dr: dr.unwrap_or(old.dr)};
    }

    pub fn set_position(&self, x: Option<GlobalCoordinateType>, y: Option<GlobalCoordinateType>, r: Option<LocalCoordinateType>) {
        let mut old = self.coordinates.lock().unwrap();
        *old = CoordinatesVelocity{x: x.unwrap_or(old.x), y: y.unwrap_or(old.y), r: r.unwrap_or(old.r), dx: old.dx, dy: old.dy, dr: old.dr};
    }

    pub fn set_coordinates(&self, new_coordinates: CoordinatesVelocity) {
        let mut old = self.coordinates.lock().unwrap();
        *old = new_coordinates;
    }

}

pub trait MobileObject {
    fn get_coordinates(&self) -> CoordinatesVelocity {
        return self.get_motion_component().get_coordinates();
    }

    fn get_motion_component(&self) -> &MotionComponent;
}
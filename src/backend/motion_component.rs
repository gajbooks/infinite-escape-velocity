use super::super::shared_types::*;
use std::sync::Mutex;

pub struct UnboundedMotionComponent {
    coordinates: Mutex<CoordinatesVelocity>,
}

impl UnboundedMotionComponent {
    pub fn new() -> UnboundedMotionComponent {
        return UnboundedMotionComponent {
            coordinates: Mutex::new(CoordinatesVelocity {
                x: 0.0,
                y: 0.0,
                r: 0.0,
                dx: 0.0,
                dy: 0.0,
                dr: 0.0,
            }),
        };
    }

    pub fn new_from_position(position: &CoordinatesRotation) -> UnboundedMotionComponent {
        return UnboundedMotionComponent {
            coordinates: Mutex::new(CoordinatesVelocity {
                x: position.x,
                y: position.y,
                r: position.r,
                dx: 0.0,
                dy: 0.0,
                dr: 0.0,
            }),
        };
    }
}

impl MotionComponent for UnboundedMotionComponent {
    fn add_velocity(&self, ax: LocalCoordinateType, ay: LocalCoordinateType) {
        {
            let mut old = self.coordinates.lock().unwrap();

            old.dx = old.dx + ax;
            old.dy = old.dy + ay;
        }
    }

    fn apply_acceleration(&self, delta_t: DeltaT, dx: LocalCoordinateType, dy: LocalCoordinateType) {
        let dvx = delta_t * dx;
        let dvy = delta_t * dy;

        self.add_velocity(dvx, dvy)
    }

    fn apply_velocity_tick(&self, delta_t: DeltaT) {
        let mut old = self.coordinates.lock().unwrap();
        let new_x = old.x + (old.dx * delta_t) as f64;
        let new_y = old.y + (old.dy * delta_t) as f64;
        let new_r = old.r + (old.dr * delta_t);
        *old = CoordinatesVelocity {
            x: new_x,
            y: new_y,
            r: new_r,
            dx: old.dx,
            dy: old.dy,
            dr: old.dr,
        };
    }

    fn get_coordinates(&self) -> CoordinatesVelocity {
        return self.coordinates.lock().unwrap().clone();
    }

    fn set_velocity(
        &self,
        dx: Option<LocalCoordinateType>,
        dy: Option<LocalCoordinateType>,
        dr: Option<LocalCoordinateType>,
    ) {
        let mut old = self.coordinates.lock().unwrap();
        *old = CoordinatesVelocity {
            x: old.x,
            y: old.y,
            r: old.r,
            dx: dx.unwrap_or(old.dx),
            dy: dy.unwrap_or(old.dy),
            dr: dr.unwrap_or(old.dr),
        };
    }

    fn set_position(
        &self,
        x: Option<GlobalCoordinateType>,
        y: Option<GlobalCoordinateType>,
        r: Option<LocalCoordinateType>,
    ) {
        let mut old = self.coordinates.lock().unwrap();
        *old = CoordinatesVelocity {
            x: x.unwrap_or(old.x),
            y: y.unwrap_or(old.y),
            r: r.unwrap_or(old.r),
            dx: old.dx,
            dy: old.dy,
            dr: old.dr,
        };
    }
}

pub struct MaximumSpeedMotionComponent {
    coordinates: Mutex<CoordinatesVelocity>,
    maximum_speed_squared: Mutex<LocalCoordinateType>,
    maximum_angular_velocity: Mutex<LocalCoordinateType>,
}

impl MaximumSpeedMotionComponent {
    pub fn new(
        maximum_speed: LocalCoordinateType,
        maximum_angular_velocity: LocalCoordinateType,
    ) -> MaximumSpeedMotionComponent {
        return MaximumSpeedMotionComponent {
            coordinates: Mutex::new(CoordinatesVelocity {
                x: 0.0,
                y: 0.0,
                r: 0.0,
                dx: 0.0,
                dy: 0.0,
                dr: 0.0,
            }),
            maximum_speed_squared: Mutex::new(maximum_speed.powi(2)),
            maximum_angular_velocity: Mutex::new(maximum_angular_velocity),
        };
    }

    pub fn new_from_position(
        position: &CoordinatesRotation,
        maximum_speed: LocalCoordinateType,
        maximum_angular_velocity: LocalCoordinateType,
    ) -> MaximumSpeedMotionComponent {
        return MaximumSpeedMotionComponent {
            coordinates: Mutex::new(CoordinatesVelocity {
                x: position.x,
                y: position.y,
                r: position.r,
                dx: 0.0,
                dy: 0.0,
                dr: 0.0,
            }),
            maximum_speed_squared: Mutex::new(maximum_speed.powi(2)),
            maximum_angular_velocity: Mutex::new(maximum_angular_velocity),
        };
    }

    fn cap_maximum_speed(&self) {
        let locked_max_speed = self.maximum_speed_squared.lock().unwrap();
        let mut speed = self.coordinates.lock().unwrap();
        let speed_squared = speed.dx.powi(2) + speed.dy.powi(2);

        {
            if speed_squared > *locked_max_speed {
                let velocity_ratio = locked_max_speed.sqrt() / speed_squared.sqrt();
                speed.dx = velocity_ratio * speed.dx;
                speed.dy = velocity_ratio * speed.dy;
            }
        }
    }

    fn cap_maximum_angular_velocity(&self) {
        let mut speed = self.coordinates.lock().unwrap();
        let dr_ratio;

        {
            let locked = self.maximum_angular_velocity.lock().unwrap();
            dr_ratio = (speed.dr / *locked).abs();
        }

        if dr_ratio > 1.0 {
            speed.dr = speed.dr / dr_ratio;
        }
    }

    pub fn set_maximum_speed(&self, maximum_speed: LocalCoordinateType) {
        {
            let mut locked = self.maximum_speed_squared.lock().unwrap();
            *locked = maximum_speed.powi(2);
        }

        self.cap_maximum_speed();
    }
}

impl MotionComponent for MaximumSpeedMotionComponent {
    fn add_velocity(&self, ax: LocalCoordinateType, ay: LocalCoordinateType) {
        {
            let mut old = self.coordinates.lock().unwrap();

            let subtract_x = ax + old.dx - old.dy;
            let subtract_y = old.dx - old.dy - ay;
            old.dx = old.dx + ax - subtract_x;
            old.dy = old.dy + ay - subtract_y;
        }

        self.cap_maximum_speed();
    }

    fn apply_acceleration(&self, delta_t: DeltaT, dx: LocalCoordinateType, dy: LocalCoordinateType) {
        let dvx = delta_t * dx;
        let dvy = delta_t * dy;

        self.add_velocity(dvx, dvy)
    }

    fn apply_velocity_tick(&self, delta_t: DeltaT) {
        let mut old = self.coordinates.lock().unwrap();
        let new_x = old.x + (old.dx * delta_t) as f64;
        let new_y = old.y + (old.dy * delta_t) as f64;
        let new_r = old.r + (old.dr * delta_t);
        *old = CoordinatesVelocity {
            x: new_x,
            y: new_y,
            r: new_r,
            dx: old.dx,
            dy: old.dy,
            dr: old.dr,
        };
    }

    fn get_coordinates(&self) -> CoordinatesVelocity {
        return self.coordinates.lock().unwrap().clone();
    }

    fn set_velocity(
        &self,
        dx: Option<LocalCoordinateType>,
        dy: Option<LocalCoordinateType>,
        dr: Option<LocalCoordinateType>,
    ) {
        {
            let mut old = self.coordinates.lock().unwrap();
            let new_dx = dx.unwrap_or(old.dx);
            let new_dy = dy.unwrap_or(old.dy);
            let new_dr = dr.unwrap_or(old.dr);

            *old = CoordinatesVelocity {
                x: old.x,
                y: old.y,
                r: old.r,
                dx: new_dx,
                dy: new_dy,
                dr: new_dr,
            };
        }

        self.cap_maximum_speed();
        self.cap_maximum_angular_velocity();
    }

    fn set_position(
        &self,
        x: Option<GlobalCoordinateType>,
        y: Option<GlobalCoordinateType>,
        r: Option<LocalCoordinateType>,
    ) {
        let mut old = self.coordinates.lock().unwrap();
        *old = CoordinatesVelocity {
            x: x.unwrap_or(old.x),
            y: y.unwrap_or(old.y),
            r: r.unwrap_or(old.r),
            dx: old.dx,
            dy: old.dy,
            dr: old.dr,
        };
    }
}

pub trait MotionComponent {
    fn add_velocity(&self, ax: LocalCoordinateType, ay: LocalCoordinateType);

    fn apply_acceleration(&self, delta_t: DeltaT, dx: LocalCoordinateType, dy: LocalCoordinateType);

    fn apply_velocity_tick(&self, delta_t: DeltaT);

    fn get_coordinates(&self) -> CoordinatesVelocity;

    fn set_velocity(
        &self,
        dx: Option<LocalCoordinateType>,
        dy: Option<LocalCoordinateType>,
        dr: Option<LocalCoordinateType>,
    );

    fn set_position(
        &self,
        x: Option<GlobalCoordinateType>,
        y: Option<GlobalCoordinateType>,
        r: Option<LocalCoordinateType>,
    );
}

pub trait MobileObject {
    fn get_coordinates(&self) -> CoordinatesVelocity {
        return self.get_motion_component().get_coordinates();
    }

    fn get_motion_component(&self) -> &dyn MotionComponent;
}

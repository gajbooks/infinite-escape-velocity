use super::super::shared_types::*;
use euclid::*;
use std::sync::Mutex;

pub struct UnboundedMotionComponent {
    coordinates: Mutex<CoordinatesVelocity>,
}

impl UnboundedMotionComponent {
    pub fn new() -> UnboundedMotionComponent {
        return UnboundedMotionComponent {
            coordinates: Mutex::new(CoordinatesVelocity {
                location: Coordinates::new(0.0, 0.0),
                rotation: Rotation::radians(0.0),
                velocity: Velocity::new(0.0, 0.0),
                angular_velocity: AngularVelocity::radians(0.0)
            }),
        };
    }

    pub fn new_from_position(position: &CoordinatesRotation) -> UnboundedMotionComponent {
        return UnboundedMotionComponent {
            coordinates: Mutex::new(CoordinatesVelocity {
                location: position.location,
                rotation: position.rotation,
                velocity: Velocity::new(0.0, 0.0),
                angular_velocity: AngularVelocity::radians(0.0)
            }),
        };
    }
}

impl MotionComponent for UnboundedMotionComponent {
    fn add_velocity(&self, added_velocity: Velocity) {
        {
            let mut old = self.coordinates.lock().unwrap();
            old.velocity = old.velocity + added_velocity;
        }
    }

    fn apply_acceleration(&self, delta_t: DeltaT, added_acceleration: Acceleration) {
        let dv = added_acceleration * DeltaTA::new(delta_t.get());

        self.add_velocity(dv.into());
    }

    fn apply_velocity_tick(&self, delta_t: DeltaT) {
        let mut old = self.coordinates.lock().unwrap();
        let new_location = old.location + (old.velocity * delta_t).to_f64();
        let new_rotation = old.rotation + (old.angular_velocity * delta_t.get());
        *old = CoordinatesVelocity {
            location: new_location,
            rotation: new_rotation,
            velocity: old.velocity,
            angular_velocity: old.angular_velocity
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
        let new_velocity = Velocity::new(dx.unwrap_or(old.velocity.x), dy.unwrap_or(old.velocity.y));
        let new_angular_velocity = AngularVelocity::radians(dr.unwrap_or(old.angular_velocity.get()));

        *old = CoordinatesVelocity {
            location: old.location,
            rotation: old.rotation,
            velocity: new_velocity,
            angular_velocity: new_angular_velocity
        };
    }

    fn set_position(
        &self,
        x: Option<GlobalCoordinateType>,
        y: Option<GlobalCoordinateType>,
        r: Option<LocalCoordinateType>,
    ) {
        let mut old = self.coordinates.lock().unwrap();

        let new_location = Coordinates::new(x.unwrap_or(old.location.x), y.unwrap_or(old.location.y));
        let new_rotation = Rotation::radians(r.unwrap_or(old.rotation.get()));

        *old = CoordinatesVelocity {
            location: new_location,
            rotation: new_rotation,
            velocity: old.velocity,
            angular_velocity: old.angular_velocity
        };
    }
}

pub struct MaximumSpeedMotionComponent {
    coordinates: Mutex<CoordinatesVelocity>,
    maximum_speed: Mutex<LocalCoordinateType>,
    maximum_angular_velocity: Mutex<LocalCoordinateType>,
}

impl MaximumSpeedMotionComponent {
    pub fn new(
        maximum_speed: LocalCoordinateType,
        maximum_angular_velocity: LocalCoordinateType,
    ) -> MaximumSpeedMotionComponent {
        return MaximumSpeedMotionComponent {
            coordinates: Mutex::new(CoordinatesVelocity {
                location: Coordinates::new(0.0, 0.0),
                rotation: Rotation::radians(0.0),
                velocity: Velocity::new(0.0, 0.0),
                angular_velocity: AngularVelocity::radians(0.0)
            }),
            maximum_speed: Mutex::new(maximum_speed),
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
                location: position.location,
                rotation: position.rotation,
                velocity: Velocity::new(0.0, 0.0),
                angular_velocity: AngularVelocity::radians(0.0)
            }),
            maximum_speed: Mutex::new(maximum_speed.powi(2)),
            maximum_angular_velocity: Mutex::new(maximum_angular_velocity),
        };
    }

    fn cap_maximum_speed(&self) {
        let locked_max_speed = self.maximum_speed.lock().unwrap();
        let mut speed = self.coordinates.lock().unwrap();
        speed.velocity = speed.velocity.clamp_length(0.0, *locked_max_speed);
    }

    fn cap_maximum_angular_velocity(&self) {
        let mut speed = self.coordinates.lock().unwrap();
        let dr_ratio;

        {
            let locked = self.maximum_angular_velocity.lock().unwrap();
            dr_ratio = (speed.angular_velocity.get() / *locked).abs();
        }

        if dr_ratio > 1.0 {
            speed.angular_velocity = speed.angular_velocity / dr_ratio;
        }
    }

    pub fn set_maximum_speed(&self, maximum_speed: LocalCoordinateType) {
        {
            let mut locked = self.maximum_speed.lock().unwrap();
            *locked = maximum_speed;
        }

        self.cap_maximum_speed();
    }
}

impl MotionComponent for MaximumSpeedMotionComponent {
    fn add_velocity(&self, added_velocity: Velocity) {
        {
            let mut old = self.coordinates.lock().unwrap();
            old.velocity = old.velocity + added_velocity;
        }

        self.cap_maximum_speed();
    }

    fn apply_acceleration(&self, delta_t: DeltaT, added_acceleration: Acceleration) {
        let dv = added_acceleration * DeltaTA::new(delta_t.get());

        self.add_velocity(dv);
    }

    fn apply_velocity_tick(&self, delta_t: DeltaT) {
        let mut old = self.coordinates.lock().unwrap();
        let new_location = old.location + (old.velocity * delta_t).to_f64();
        let new_rotation = old.rotation + (old.angular_velocity * delta_t.get());
        *old = CoordinatesVelocity {
            location: new_location,
            rotation: new_rotation,
            velocity: old.velocity,
            angular_velocity: old.angular_velocity
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
            let new_velocity = Velocity::new(dx.unwrap_or(old.velocity.x), dy.unwrap_or(old.velocity.y));
            let new_angular_velocity = AngularVelocity::radians(dr.unwrap_or(old.angular_velocity.get()));
    
            *old = CoordinatesVelocity {
                location: old.location,
                rotation: old.rotation,
                velocity: new_velocity,
                angular_velocity: new_angular_velocity
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

        let new_location = Coordinates::new(x.unwrap_or(old.location.x), y.unwrap_or(old.location.y));
        let new_rotation = Rotation::radians(r.unwrap_or(old.rotation.get()));

        *old = CoordinatesVelocity {
            location: new_location,
            rotation: new_rotation,
            velocity: old.velocity,
            angular_velocity: old.angular_velocity
        };
    }
}

pub trait MotionComponent {
    fn add_velocity(&self, added_velocity: Velocity);

    fn apply_acceleration(&self, delta_t: DeltaT, added_acceleration: Acceleration);

    fn apply_acceleration_along_pointing_direction(&self, delta_t: DeltaT, acceleration: LocalCoordinateType) {
        let coordinates = self.get_coordinates();
        let coordinate_acceleration = Acceleration::from_angle_and_length(coordinates.rotation, acceleration);

        self.apply_acceleration(delta_t, coordinate_acceleration);
    }

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

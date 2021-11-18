use super::super::shared_types::*;
use euclid::*;
use std::sync::Mutex;

pub struct MaximumSpeedMotionComponent {
    coordinates: Mutex<CoordinatesVelocity>,
    acceleration: Mutex<Acceleration>,
    maximum_speed: Mutex<LocalCoordinateType>,
    maximum_acceleration: Mutex<LocalCoordinateType>,
    maximum_angular_velocity: Mutex<LocalCoordinateType>,
}

impl MaximumSpeedMotionComponent {
    pub fn new(
        maximum_speed: LocalCoordinateType,
        maximum_acceleration: LocalCoordinateType,
        maximum_angular_velocity: LocalCoordinateType,
    ) -> MaximumSpeedMotionComponent {
        return MaximumSpeedMotionComponent {
            coordinates: Mutex::new(CoordinatesVelocity {
                location: Coordinates::default(),
                rotation: Rotation::default(),
                velocity: Velocity::default(),
                angular_velocity: AngularVelocity::default()
            }),
            acceleration: Mutex::new(Acceleration::default()),
            maximum_speed: Mutex::new(maximum_speed),
            maximum_acceleration: Mutex::new(maximum_acceleration),
            maximum_angular_velocity: Mutex::new(maximum_angular_velocity),
        };
    }

    pub fn new_from_position(
        position: &CoordinatesRotation,
        maximum_speed: LocalCoordinateType,
        maximum_acceleration: LocalCoordinateType,
        maximum_angular_velocity: LocalCoordinateType,
    ) -> MaximumSpeedMotionComponent {
        return MaximumSpeedMotionComponent {
            coordinates: Mutex::new(CoordinatesVelocity {
                location: position.location,
                rotation: position.rotation,
                velocity: Velocity::default(),
                angular_velocity: AngularVelocity::default()
            }),
            maximum_speed: Mutex::new(maximum_speed),
            acceleration: Mutex::new(Acceleration::default()),
            maximum_acceleration: Mutex::new(maximum_acceleration),
            maximum_angular_velocity: Mutex::new(maximum_angular_velocity),
        };
    }

    fn cap_maximum_speed(&self) {
        let locked_max_speed = self.maximum_speed.lock().unwrap();
        let mut speed = self.coordinates.lock().unwrap();
        speed.velocity = speed.velocity.clamp_length(0.0, *locked_max_speed);
    }

    fn cap_maximum_acceleration(&self) {
        let maximum_acceleration = self.maximum_acceleration.lock().unwrap();
        let mut acceleration = self.acceleration.lock().unwrap();
        *acceleration = acceleration.clamp_length(0.0, *maximum_acceleration);
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

    pub fn set_maximum_acceleration(&self, maximum_acceleration: LocalCoordinateType) {
        {
            let mut locked = self.maximum_acceleration.lock().unwrap();
            *locked = maximum_acceleration;
        }

        self.cap_maximum_acceleration();
    }
}

impl MobileObject for MaximumSpeedMotionComponent {
    fn add_velocity(&self, added_velocity: Velocity) {
        {
            let mut old = self.coordinates.lock().unwrap();
            old.velocity = old.velocity + added_velocity;
        }

        self.cap_maximum_speed();
    }

    fn add_acceleration(&self, added_acceleration: Acceleration) {
        {
            let mut acceleration = self.acceleration.lock().unwrap();
            *acceleration += added_acceleration;
        }

        self.cap_maximum_acceleration();
    }

    fn apply_velocity_tick(&self, delta_t: DeltaT) {
        let mut coordinates = self.coordinates.lock().unwrap();
        let mut acceleration = self.acceleration.lock().unwrap();
        let angular_velocity = coordinates.angular_velocity;

        let velocity = coordinates.velocity + (*acceleration * delta_t_to_delta_t_a(delta_t));

        let new_location = coordinates.location + (velocity * delta_t).to_f64();
        let new_rotation = coordinates.rotation + (angular_velocity * delta_t.get());

        *coordinates = CoordinatesVelocity {
            location: new_location,
            rotation: new_rotation,
            velocity: velocity,
            angular_velocity: angular_velocity
        };
        *acceleration = Acceleration::default();
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

pub trait MobileObject: Sync + Send {
    fn add_velocity(&self, added_velocity: Velocity);

    fn add_acceleration(&self, added_acceleration: Acceleration);

    fn add_acceleration_along_pointing_direction(&self, acceleration: LocalCoordinateType) {
        let coordinates = self.get_coordinates();
        let coordinate_acceleration = Acceleration::from_angle_and_length(coordinates.rotation, acceleration);

        self.add_acceleration(coordinate_acceleration);
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

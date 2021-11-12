use std::sync::atomic::*;
use euclid::*;

pub type IdType = u32;
pub type AtomicIdType = AtomicU32;
pub type GlobalCoordinateType = f64;
pub type LocalCoordinateType = f32;
pub type DeltaTimeType = f32;

pub struct WorldCoordinates;
pub struct RotationCoordinates;
pub struct VelocityCoordinates;
pub struct AccelerationCoordinates;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct ShipTypeData {
    pub namespace: IdType,
    pub id: IdType
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct StaticTypeData {
    pub namespace: IdType,
    pub id: IdType
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum ObjectType {
    NonWorld(),
    Static(StaticTypeData),
    Ship(ShipTypeData)
}

pub type AABB = Box2D<GlobalCoordinateType, WorldCoordinates>;

pub type Coordinates = Point2D<GlobalCoordinateType, WorldCoordinates>;
pub type Velocity = Vector2D<LocalCoordinateType, VelocityCoordinates>;
pub type Acceleration = Vector2D<LocalCoordinateType, AccelerationCoordinates>;
pub type Rotation = Angle<LocalCoordinateType>;
pub type AngularVelocity = Angle<LocalCoordinateType>;
pub type Radius = Length<GlobalCoordinateType, WorldCoordinates>;

pub type DeltaTA = Scale<DeltaTimeType, AccelerationCoordinates, VelocityCoordinates>;
pub type DeltaT = Scale<DeltaTimeType, VelocityCoordinates, WorldCoordinates>;

#[derive(Clone)]
pub struct CoordinatesRotation {
    pub location: Coordinates,
    pub rotation: Rotation
}

#[derive(Clone)]
pub struct CoordinatesVelocity {
    pub location: Coordinates,
    pub rotation: Rotation,
    pub velocity: Velocity,
    pub angular_velocity: AngularVelocity
}
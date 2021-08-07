use std::sync::atomic::*;

pub type IdType = u32;
pub type AtomicIdType = AtomicU32;
pub type GlobalCoordinateType = f64;
pub type LocalCoordinateType = f32;
pub type DeltaT = f32;

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

#[derive(Clone)]
pub struct AABB {
    pub x1: GlobalCoordinateType,
    pub y1: GlobalCoordinateType,
    pub x2: GlobalCoordinateType,
    pub y2: GlobalCoordinateType
}

#[derive(Clone)]
pub struct Coordinates {
    pub x: GlobalCoordinateType,
    pub y: GlobalCoordinateType
}

#[derive(Clone)]
pub struct CoordinatesRotation {
    pub x: GlobalCoordinateType,
    pub y: GlobalCoordinateType,
    pub r: LocalCoordinateType
}

#[derive(Clone)]
pub struct CoordinatesVelocity {
    pub x: GlobalCoordinateType,
    pub y: GlobalCoordinateType,
    pub r: LocalCoordinateType,
    pub dx: LocalCoordinateType,
    pub dy: LocalCoordinateType,
    pub dr: LocalCoordinateType
}
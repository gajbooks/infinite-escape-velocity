use std::hash::*;

pub type HashCoordinateType = i32;

#[derive(Hash, PartialEq, Eq)]
pub struct HashCoordinates{
    pub x: HashCoordinateType,
    pub y: HashCoordinateType
}
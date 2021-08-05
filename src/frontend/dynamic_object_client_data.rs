use super::super::shared_types::*;

pub struct DynamicObjectClientData {
    pub x: f64,
    pub y: f64,
    pub rotation: f32,
    pub vx: f32,
    pub vy: f32,
    pub angular_velocity: f32,
    pub object_type: IdType
}
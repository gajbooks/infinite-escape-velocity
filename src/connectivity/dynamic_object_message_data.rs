use super::super::shared_types::*;

pub struct DynamicObjectMessageData {
    pub x: f64,
    pub y: f64,
    pub rotation: f32,
    pub vx: f32,
    pub vy: f32,
    pub angular_velocity: f32,
    pub object_type: IdType,
    pub id: IdType
}

pub struct DynamicObjectCreationData {
    pub id: IdType
}

pub struct DynamicObjectDestructionData {
    pub id: IdType
}
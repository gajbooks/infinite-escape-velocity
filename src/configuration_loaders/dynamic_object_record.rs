

pub struct MovementParameters {
    pub maximum_speed: f32,
    pub maximum_acceleration: f32,
    pub maximum_angular_velocity: f32
}

pub struct CirleParameters {
    pub radius: f32
}

pub struct RoundedTubeParameters {
    pub radius: f32,
    pub length: f32
}

pub struct CollisionParameters {
    pub circle: Option<CirleParameters>,
    pub rounded_tube: Option<RoundedTubeParameters>
}

pub struct MunitionParameters {
    pub damage: f32
}

pub struct DynamicObjectRecord {
    pub author: String,
    pub object_type: String,
    pub object_version: u32,
    pub movement_parameters: Option<MovementParameters>,
    pub collision_parameters: Option<CollisionParameters>,
    pub munition_parameters: Option<MunitionParameters>
}
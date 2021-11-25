use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct MovementParameters {
    pub maximum_speed: f32,
    pub maximum_acceleration: f32,
    pub maximum_angular_velocity: f32
}

#[derive(Clone, Deserialize)]
pub struct CirleParameters {
    pub radius: f32
}

#[derive(Clone, Deserialize)]
pub struct RoundedTubeParameters {
    pub radius: f32,
    pub length: f32
}

#[derive(Clone, Deserialize)]
pub struct CollisionParameters {
    pub circle: Option<CirleParameters>,
    pub rounded_tube: Option<RoundedTubeParameters>
}

#[derive(Clone, Deserialize)]
pub struct MunitionParameters {
    pub damage: f32
}

#[derive(Clone, Deserialize)]
pub struct LifeParameters {
    pub structure_points: f32,
    pub shield_points: f32
}

#[derive(Clone, Deserialize)]
pub struct GraphicsParameters {
    pub simple: Option<SimpleTextureGraphic>
}

#[derive(Clone, Deserialize)]
pub struct SimpleTextureGraphic {
    pub filename: String
}

#[derive(Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct DynamicObjectTypeParameters {
    pub author: String,
    pub object_type: String
}

#[derive(Clone, Deserialize)]
pub struct DynamicObjectRecord {
    pub object_type: DynamicObjectTypeParameters,
    pub object_version: u32,
    pub movement_parameters: Option<MovementParameters>,
    pub collision_parameters: Option<CollisionParameters>,
    pub munition_parameters: Option<MunitionParameters>,
    pub graphics_parameters: Option<GraphicsParameters>,
    pub life_parameters: Option<LifeParameters>
}
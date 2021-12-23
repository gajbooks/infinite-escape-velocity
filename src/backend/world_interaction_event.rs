use super::super::shared_types::*;
use super::super::configuration_loaders::dynamic_object_record::*;

pub struct ObjectSpawnParameters {
    pub object_type: DynamicObjectTypeParameters,
    pub position: CoordinatesRotation
}

pub enum WorldInteractionEvent {
    RemoveObject(IdType),
    SpawnObject(ObjectSpawnParameters)
}

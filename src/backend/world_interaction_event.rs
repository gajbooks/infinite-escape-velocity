use super::super::shared_types::*;
use super::super::configuration_loaders::dynamic_object_record::*;

pub struct ObjectSpawnParameters {
    object_type: DynamicObjectTypeParameters,
    position: CoordinatesRotation
}

pub enum WorldInteractionEvent {
    RemoveObject(IdType),
    SpawnObject(ObjectSpawnParameters)
}
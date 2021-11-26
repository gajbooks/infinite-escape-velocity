use super::super::configuration_loaders::object_type_map::*;
use super::super::configuration_loaders::dynamic_object_record::*;
use super::super::configuration_loaders::dynamic_object_configuration::*;
use std::sync::*;
use super::unique_object::*;
use super::unique_id_allocator::*;
use super::super::shared_types::*;

pub struct WorldObjectConstructor {
    object_type_map: Arc<ObjectTypeMap>,
    dynamic_object_configuration: Arc<DynamicObjectConfiguration>,
    unique_id_allocator: Arc<UniqueIdAllocator>
}

impl WorldObjectConstructor {
    pub fn new(
        object_type_map: Arc<ObjectTypeMap>,
        dynamic_object_configuration: Arc<DynamicObjectConfiguration>,
        unique_id_allocator: Arc<UniqueIdAllocator>
        ) -> WorldObjectConstructor {
            WorldObjectConstructor {
                object_type_map: object_type_map,
                dynamic_object_configuration: dynamic_object_configuration,
                unique_id_allocator: unique_id_allocator
            }
    }

    pub fn construct_from_type<T: FromPrototype>(&self, object_type: &DynamicObjectTypeParameters, position: CoordinatesRotation) -> Option<Arc<dyn UniqueObject + Send + Sync>> {
        let unique_id = self.unique_id_allocator.new_allocated_id();
        let mapped_type = match self.object_type_map.object_type_parameters_to_object_type(object_type) {
            Ok(has) => has,
            Err(()) => {
                println!("Object type not found: {:?}", object_type);
                return None;
            }
        };

        let type_record = match self.dynamic_object_configuration.get(object_type) {
            Some(has) => has,
            None => {
                println!("Object prototype not found: {:?}", object_type);
                return None;
            }
        };

        match T::from_prototype(&type_record, mapped_type, position, unique_id) {
            Ok(compatible) => Some(compatible),
            Err(()) => {
                println!("Could not construct object from prototype: {:?} {:?}", type_record, object_type);
                None
            }
        }
    }
}

pub trait FromPrototype: UniqueObject + Send + Sync {
    fn from_prototype(object_record: &DynamicObjectRecord, object_type: ObjectType, position: CoordinatesRotation, id: ReturnableId) -> Result<Arc<dyn UniqueObject + Send + Sync>, ()>;
}
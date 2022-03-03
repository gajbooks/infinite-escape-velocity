/*
    This file is part of Infinite Escape Velocity.

    Infinite Escape Velocity is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Infinite Escape Velocity is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Infinite Escape Velocity.  If not, see <https://www.gnu.org/licenses/>.
*/

use super::super::configuration_loaders::object_type_map::*;
use super::super::configuration_loaders::object_configuration_record::*;
use super::super::configuration_loaders::object_configuration::*;
use std::sync::*;
use super::unique_object_storage::{unique_object::*, unique_id_allocator::*};
use super::super::shared_types::*;

pub struct WorldObjectConstructor {
    object_type_map: Arc<ObjectTypeMap>,
    dynamic_object_configuration: Arc<ObjectConfigurationMap>,
    unique_id_allocator: Arc<UniqueIdAllocator>
}

impl WorldObjectConstructor {
    pub fn new(
        object_type_map: Arc<ObjectTypeMap>,
        dynamic_object_configuration: Arc<ObjectConfigurationMap>,
        unique_id_allocator: Arc<UniqueIdAllocator>
        ) -> WorldObjectConstructor {
            WorldObjectConstructor {
                object_type_map: object_type_map,
                dynamic_object_configuration: dynamic_object_configuration,
                unique_id_allocator: unique_id_allocator
            }
    }

    pub fn construct_from_type<T: FromPrototype>(&self, object_type: &ObjectTypeParameters, position: CoordinatesRotation) -> Option<Arc<dyn UniqueObject + Send + Sync>> {
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
    fn from_prototype(object_record: &ObjectConfigurationRecord, object_type: ObjectType, position: CoordinatesRotation, id: ReturnableId) -> Result<Arc<dyn UniqueObject + Send + Sync>, ()>;
}
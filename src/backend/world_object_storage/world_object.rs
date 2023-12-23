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

use std::sync::Arc;

use serde::{Serialize, Deserialize};

use crate::backend::world_objects::object_properties::collision_property::CollidableObject;

use crate::backend::world_objects::ship::{Ship, ShipSerializationData};
use crate::shared_types::*;

use super::ephemeral_id_allocator::IdAllocatorType;

#[derive(Serialize, Deserialize)]
pub enum WorldObjectSerializationData {
    None, // Empty data, do not serialize or deserialize
    Ship(ShipSerializationData)
}

impl WorldObjectSerializationData {
    fn deserialize_object_with_id(&self, id: IdType) -> Option<Arc<dyn WorldObject>> {
        match self {
            WorldObjectSerializationData::None => None,
            WorldObjectSerializationData::Ship(data) => Some(Arc::new(Ship::new_from_serialized(data, id)))
        }
    }

    pub fn deserialize_object(&self, allocator: IdAllocatorType) -> Option<Arc<dyn WorldObject>> {
        self.deserialize_object_with_id(allocator.new_id())
    }

    pub fn serialize_object<'a>(from: &'a dyn WorldObject) -> Option<WorldObjectSerializationData> {
        let data = from.get_serialization_data();
        match data {
            WorldObjectSerializationData::None => None,
            _ => Some(data)
        }
    }

    pub fn deserialize_list<'a, 'b, S: Iterator<Item = &'a WorldObjectSerializationData> + 'a>(from: S, allocator: IdAllocatorType) -> impl Iterator<Item = Arc<dyn WorldObject>> + 'a {
        from.filter_map(move |world_object| world_object.deserialize_object_with_id(allocator.new_id()))
    }

    pub fn serialize_list<'a, T: Iterator<Item = &'a dyn WorldObject> + 'a>(from: T) -> impl Iterator<Item = WorldObjectSerializationData> + 'a {
        from.filter_map(|x| WorldObjectSerializationData::serialize_object(x))
    }
}

pub trait WorldObject: Send + Sync {
    fn get_id(&self) -> IdType;
    fn get_type(&self) -> ObjectType;
    fn get_collision_property(&self) -> Option<&dyn CollidableObject> {
        return None;
    }
    fn get_serialization_data(&self) -> WorldObjectSerializationData;
    fn tick(&self, _delta_t: DeltaT);
}
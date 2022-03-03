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

use super::super::shared_types::*;
use dashmap::DashMap;
use fxhash::FxBuildHasher;
use super::object_configuration_record::*;

pub struct ObjectTypeMap {
    text_to_type: DashMap<ObjectTypeParameters, ObjectType, FxBuildHasher>,
    type_to_text: DashMap<ObjectType, ObjectTypeParameters, FxBuildHasher>,
    current_object_id: AtomicIdType
}

impl ObjectTypeMap {
    pub fn new() -> ObjectTypeMap {
        return ObjectTypeMap {
            text_to_type: DashMap::with_hasher(FxBuildHasher::default()),
            type_to_text: DashMap::with_hasher(FxBuildHasher::default()),
            current_object_id: AtomicIdType::new(0)};
    }

    pub fn object_type_parameters_to_object_type(&self, object_type: &ObjectTypeParameters) -> Result<ObjectType, ()> {
        return match self.text_to_type.get(object_type) {
            Some(has) => Ok(has.value().clone()),
            None => Err(())
        }
    }

    pub fn object_type_to_object_type_parameters(&self, object_type: &ObjectType) -> Result<Option<ObjectTypeParameters>, ()> {
        match object_type {
            ObjectType::WorldObject(_type_data) => {
                match self.type_to_text.get(object_type) {
                    Some(has) => Ok(Some(has.value().clone())),
                    None => Err(())
                }
            },
            _ => Ok(None)
        }
    }

    pub fn add_object_type(&self, object_type: &ObjectTypeParameters) -> ObjectType {
        match self.object_type_parameters_to_object_type(object_type) {
            Ok(has) => {
                println!("Attempted to register duplicate object type with author {} and type {}", object_type.author, object_type.object_type);
                return has;
            },
            Err(()) => ()
        };

        let new_id = self.current_object_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.text_to_type.insert(object_type.to_owned(), ObjectType::WorldObject(new_id));
        self.type_to_text.insert(ObjectType::WorldObject(new_id), object_type.to_owned());

        return ObjectType::WorldObject(new_id);
    }
}
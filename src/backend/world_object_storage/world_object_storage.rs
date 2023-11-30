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
use dashmap::DashMap;
use crate::shared_types::*;
use crate::backend::world_object_storage::world_object::*;
use crate::backend::shrink_storage::ImmutableShrinkable;

use super::{ephemeral_id_allocator::{EphemeralIdAllocator, IdAllocatorType}, cleanup_predicate::{self, retain_world_objects}};

pub struct WorldObjectStorage {
    objects: DashMap<IdType, Arc<dyn WorldObject>>
}

impl WorldObjectStorage {
    pub fn new() -> WorldObjectStorage {
        return WorldObjectStorage {objects: DashMap::new()};
    }

    pub fn add(&self, new: Arc<dyn WorldObject>) {

        match self.objects.entry(new.get_id()) {
            dashmap::mapref::entry::Entry::Occupied(_has) => (),
            dashmap::mapref::entry::Entry::Vacant(not) => {
                not.insert(new);
            }
        };
    }

    pub fn cleanup(&self) {
        self.objects.retain(|_, x| retain_world_objects(x.as_ref()));
    }

    pub fn all_objects(&self) -> Vec<Arc<dyn WorldObject>> {
        self.objects.iter().map(|x| x.value().clone()).collect()
    }

    pub fn serialize(&self) -> Vec<WorldObjectSerializationData> {
        WorldObjectSerializationData::serialize_list(self.all_objects().iter().map(|x| x.as_ref())).collect()
    }

    pub fn deserialize<'a>(iter: impl Iterator<Item = &'a WorldObjectSerializationData> + 'a, allocator: IdAllocatorType) -> WorldObjectStorage {
        WorldObjectStorage { objects: DashMap::from_iter(WorldObjectSerializationData::deserialize_list(iter, allocator).map(|x| (x.get_id(), x))) }
    }
}
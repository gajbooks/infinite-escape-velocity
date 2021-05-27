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

use fxhash::FxBuildHasher;
use dashmap::DashMap;
use super::identifiable_object::*;
use super::shrink_storage;

pub trait StoredObject {
    fn get_id(&self) -> IdType;
    fn process_messages(&self);
    // Placeholder for now, later will be able to serialize and deserialize
}

pub struct UniqueObjectStorage {
    objects: DashMap<IdType, Box<dyn StoredObject + Send>, FxBuildHasher>,
}

impl UniqueObjectStorage {
    pub fn new() -> UniqueObjectStorage {
        return UniqueObjectStorage {objects: DashMap::with_hasher(FxBuildHasher::default())};
    }

    pub fn add(&self, new: Box<dyn StoredObject + Send>) -> Result<(), Box<dyn StoredObject>> {

        match self.objects.entry(new.get_id()) {
            dashmap::mapref::entry::Entry::Occupied(_has) => {
                return Err(new);
            },
            dashmap::mapref::entry::Entry::Vacant(not) => {
                not.insert(new);
                return Ok(());
            }
        };
    }

    pub fn remove(&self, del: IdType) {
        self.objects.remove(&del);
        shrink_storage!(self.objects);
    }

    // Replace with Tokio or something
    pub fn process_object_messages(&self) {
        for i in &self.objects {
            i.process_messages();
        }
    }
}
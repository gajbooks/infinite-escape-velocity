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
use std::sync::Arc;
use rayon::prelude::*;
use super::super::super::shared_types::*;
use super::unique_object::*;

pub struct UniqueObjectStorage {
    objects: DashMap<IdType, Arc<dyn UniqueObject + Sync + Send>, FxBuildHasher>
}

impl UniqueObjectStorage {
    pub fn new() -> UniqueObjectStorage {
        return UniqueObjectStorage {objects: DashMap::with_hasher(FxBuildHasher::default())};
    }

    pub fn add(&self, new: Arc<dyn UniqueObject + Sync + Send>) {

        match self.objects.entry(new.get_id()) {
            dashmap::mapref::entry::Entry::Occupied(_has) => (),
            dashmap::mapref::entry::Entry::Vacant(not) => {
                not.insert(new);
            }
        };
    }

    pub fn remove(&self, del: IdType) -> bool {
        let existed = self.objects.remove(&del).is_some();
        crate::shrink_storage!(self.objects);
        return existed;
    }

    pub fn get_by_id(&self, id: IdType) -> Option<Arc<dyn UniqueObject + Sync + Send>>{
        return match self.objects.get(&id) {
            None => None,
            Some(has) => Some(has.value().clone())
        };
    }

    pub fn all_objects(&self) -> Vec<Arc<dyn UniqueObject + Sync + Send>> {
        self.objects.iter().map(|x| x.clone()).collect()
    }
}
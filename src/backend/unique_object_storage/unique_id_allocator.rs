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

use std::sync::atomic::*;
use dashmap::DashSet;
use std::sync::*;
use super::super::super::shared_types::*;
use fxhash::FxBuildHasher;

impl Drop for ReturnableId {
    fn drop(&mut self) {
        self.allocator.drop_allocated_id(self.id);
    }
}

pub struct ReturnableId {
    pub id: IdType,
    allocator: Arc<UniqueIdInternal>
}

impl ReturnableId {
    fn new(id: IdType, allocator: Arc<UniqueIdInternal>) -> ReturnableId {
        return ReturnableId{id: id, allocator: allocator};
    }
}

pub struct UniqueIdAllocator {
    internal: Arc<UniqueIdInternal>
}

impl UniqueIdAllocator {
    pub fn new_allocated_id(&self) -> ReturnableId {
        self.internal.new_allocated_id(self.internal.clone())
    }

    pub fn new() -> UniqueIdAllocator {
        UniqueIdAllocator{internal: Arc::new(UniqueIdInternal::new())}
    }
}

pub struct UniqueIdInternal {
    id_tracker: AtomicIdType,
    allocated_ids: DashSet<IdType, FxBuildHasher>
}

impl UniqueIdInternal {
    pub fn drop_allocated_id(&self, id: IdType) {
        self.allocated_ids.remove(&id);

        crate::shrink_storage!(self.allocated_ids);
    }

    fn new_allocated_id(&self, self_ref: Arc<UniqueIdInternal>) -> ReturnableId {
        let mut id = self.id_tracker.fetch_add(1, Ordering::Relaxed);
        while !self.allocated_ids.insert(id) {
            id = self.id_tracker.fetch_add(1, Ordering::Relaxed);
        }

        ReturnableId::new(id, self_ref)
    }

    fn new() -> UniqueIdInternal {
        UniqueIdInternal{id_tracker: AtomicIdType::new(0), allocated_ids: DashSet::with_hasher(FxBuildHasher::default())}
    }
}
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

use dashmap::DashSet;

use crate::backend::shrink_storage::*;
use crate::backend::world_object_storage::world_object::WorldObject;
use crate::shared_types::*;
use crate::backend::shape::*;

pub struct AlreadyCollidedTracker {
    list: DashSet<IdType>,
}

impl AlreadyCollidedTracker {
    pub fn clear(&self) {
        self.list.shrink_storage();
        self.list.clear();
    }

    pub fn not_collided(&self, id: IdType) -> bool {
        return self.list.insert(id);
    }

    pub fn get_list(&self) -> DashSet<IdType> {
        return self.list.clone();
    }

    pub fn new() -> AlreadyCollidedTracker {
        AlreadyCollidedTracker {
            list: DashSet::new(),
        }
    }
}

pub trait CollidableObject: Send + Sync {
    fn collide_with(&self, collided_object: &dyn WorldObject) {
        if self.get_already_collided().not_collided(collided_object.get_id()) {
            self.do_collision(collided_object);
        }
    }

    fn do_collision(&self, collided_object: &dyn WorldObject);

    fn get_already_collided(&self) -> &AlreadyCollidedTracker;

    fn get_shape(&self) -> Shape;

    fn set_shape(&self, shape: Shape) -> Shape;
}
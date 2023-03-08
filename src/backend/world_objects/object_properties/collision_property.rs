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

use crate::shared_types::*;
use crate::backend::shape::*;
use std::sync::Mutex;

pub struct AlreadyCollidedTracker {
    list: Mutex<Vec<IdType>>,
}

impl AlreadyCollidedTracker {
    pub fn clear(&self) {
        let mut locked = self.list.lock().unwrap();
        crate::shrink_storage!(locked);
        locked.clear();
    }

    pub fn not_collided(&self, id: IdType) -> bool {
        let mut locked = self.list.lock().unwrap();
        match locked.contains(&id) {
            true => return false,
            false => {
                locked.push(id);
                return true;
            }
        }
    }

    pub fn get_list(&self) -> Vec<IdType> {
        return self.list.lock().unwrap().clone();
    }

    pub fn new() -> AlreadyCollidedTracker {
        AlreadyCollidedTracker {
            list: Mutex::new(Vec::new()),
        }
    }
}

pub trait CollidableObject: Send + Sync {
    fn collide_with(&self, shape: &Shape, id: IdType) {
        if self.get_already_collided().not_collided(id) {
            self.do_collision(shape, id);
        }
    }

    fn do_collision(&self, shape: &Shape, id: IdType);

    fn get_already_collided(&self) -> &AlreadyCollidedTracker;

    fn get_shape(&self) -> Shape;
}
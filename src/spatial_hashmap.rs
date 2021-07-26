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

use super::hash_coordinates::*;
use super::identifiable_object::*;
use super::shape::*;
use dashmap::DashMap;
use fxhash::FxBuildHasher;
use std::sync::Arc;
use std::sync::atomic::*;
use std::sync::Mutex;

pub trait CollidableObject {
    fn collide_with(&self, shape: &Shape, from: IdType);
    fn get_shape(&self) -> &Shape;
    fn get_id(&self) -> IdType;
}

struct ObjectWithinCell {
    pub cell: HashCoordinates,
    pub id: IdType,
    pub object: Arc<dyn CollidableObject>
}

pub struct SpatialHashmap {
    object_input_list: Mutex<Vec<ObjectWithinCell>>
}

impl SpatialHashmap {
    pub fn new() -> SpatialHashmap {
        return SpatialHashmap {
            object_input_list: Mutex::<Vec<ObjectWithinCell>>::new(Vec::new())
        };
    }

    pub fn add(&self, add: Arc<dyn CollidableObject>) -> () {
        let mut locked = self.object_input_list.lock().unwrap();

        for cell in add.get_shape().aabb_iter() {
            locked.push(ObjectWithinCell{cell: cell, id: add.get_id(), object: add.clone()})
        }
    }

    pub fn run_collisions(&self) -> () {
        let cell_offset_list: DashMap<HashCoordinates, AtomicUsize, FxBuildHasher> = DashMap::with_hasher(FxBuildHasher::default());

        let mut locked = self.object_input_list.lock().unwrap();

        for object in locked.iter() {
            match cell_offset_list.get(&object.cell) {
                Some(has) => {
                    has.fetch_add(1, Ordering::Relaxed);
                },
                None => {
                    cell_offset_list.insert(object.cell.clone(), AtomicUsize::new(0));
                }
            };
        }

        let mut list_position: usize = 0;

        for cell_object in cell_offset_list.iter() {
            list_position += cell_object.value().fetch_add(list_position, Ordering::Relaxed);
        }

        let mut object_output_list: Vec<&ObjectWithinCell> = Vec::new();

        let placeholder = match locked.first() {
            Some(has) => has,
            None => {
                // List has no first element so is empty
                return;
            }
        };

        object_output_list.resize(locked.len(), placeholder);

        for object in locked.iter() {
            let count = cell_offset_list.get(&object.cell).unwrap();
            let index = count.value().fetch_sub(1, Ordering::Relaxed);
            object_output_list[index] = &object;
        }

        let mut dedup = Vec::new();

        for index in 0..object_output_list.len() {
            let outer_object = &object_output_list[index];

            let mut inner_index = index + 1;

            if inner_index >= object_output_list.len() {
                continue;
            }

            dedup.clear();

            while  inner_index < object_output_list.len() && outer_object.cell == object_output_list[inner_index].cell {
                if outer_object.object.get_shape().collides(&object_output_list[inner_index].object.get_shape()) {
                    dedup.push(object_output_list[inner_index]);
                }

                inner_index += 1;
            }

            dedup.sort_unstable_by(|x,y| x.id.cmp(&y.id));
            dedup.dedup_by(|x,y| x.id.eq(&y.id));

            for dedup_object in &dedup {
                outer_object.object.collide_with(&dedup_object.object.get_shape(), dedup_object.id);
                dedup_object.object.collide_with(&outer_object.object.get_shape(), outer_object.id);
            }
        }

        locked.clear();
    }
}

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

use crate::{backend::world_objects::object_properties::collision_property::CollidableObject, shared_types::IdType};

use super::hash_coordinates::*;
use rayon::prelude::*;
use std::sync::Arc;
use super::super::unique_object_storage::unique_object::*;

struct ObjectWithinCell<'a> {
    pub cell: HashCoordinates,
    pub object: &'a (dyn CollidableObject),
    pub id: IdType
}

pub struct CollisionOptimizer {}

impl CollisionOptimizer {
    pub fn new() -> CollisionOptimizer {
        return CollisionOptimizer {};
    }

    pub fn run_collisions(&self, object_list: &[Arc<(dyn UniqueObject + Sync + Send)>]) -> () {
        let mut list: Vec<ObjectWithinCell> = object_list.par_iter()
        .filter_map(|object| object.get_collision_property().map(|x| (object.get_id(), x)))
        .flat_map_iter(|has_collision_property| has_collision_property.1.get_shape().aabb_iter().map(move |y| ObjectWithinCell{cell: y, id: has_collision_property.0, object: has_collision_property.1})).collect();

        let length = list.len();

        list.par_sort_unstable_by(|x, y| x.cell.cmp(&y.cell));

        list.iter().enumerate().for_each(|range| {
            let outer_object = &range.1;

            let mut inner_index = range.0 + 1;

            if inner_index >= length {
                return;
            }

            while inner_index < length && outer_object.cell == list[inner_index].cell {
                let inner_object = &list[inner_index];
                if outer_object
                    .object.get_shape()
                    .collides(&inner_object.object.get_shape())
                {
                    outer_object
                        .object.collide_with(&inner_object.object.get_shape(), inner_object.id);
                    inner_object
                        .object.collide_with(&outer_object.object.get_shape(), outer_object.id);
                }
                inner_index += 1;
            }
        });

        list.clear();
    }
}

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
use rayon::prelude::*;
use std::sync::Arc;
use super::unique_object::*;

struct ObjectWithinCell<'a> {
    pub cell: HashCoordinates,
    pub object: &'a (dyn UniqueObject + Sync + Send),
}

pub struct SpatialHashmap {}

impl SpatialHashmap {
    pub fn new() -> SpatialHashmap {
        return SpatialHashmap {};
    }

    pub fn run_collisions(&self, object_list: &[Arc<(dyn UniqueObject + Sync + Send)>]) -> () {
        let mut list: Vec<ObjectWithinCell> = object_list.par_iter().filter(|x| match x.as_collision_component() {
            Some(_x) => true,
            None => false
        }).flat_map_iter(|x| x.as_collision_component().unwrap().get_shape().aabb_iter().map(move |y| ObjectWithinCell{cell: y, object: x.as_ref()})).collect();

        let length = list.len();

        list.par_sort_unstable_by(|x, y| x.cell.cmp(&y.cell));

        list.par_iter().enumerate().for_each(|range| {
            let outer_object = &range.1;

            let mut inner_index = range.0 + 1;

            if inner_index >= length {
                return;
            }

            while inner_index < length && outer_object.cell == list[inner_index].cell {
                let inner_object = &list[inner_index];
                if outer_object
                    .object.as_collision_component().unwrap()
                    .get_shape()
                    .collides(inner_object.object.as_collision_component().unwrap().get_shape())
                {
                    outer_object
                        .object.as_collision_component().unwrap()
                        .collide_with(inner_object.object.as_collision_component().unwrap().get_shape(), inner_object.object.get_id());
                    inner_object
                        .object.as_collision_component().unwrap()
                        .collide_with(outer_object.object.as_collision_component().unwrap().get_shape(), outer_object.object.get_id());
                }

                inner_index += 1;
            }
        });

        list.clear();
    }
}

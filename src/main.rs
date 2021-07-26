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

mod spatial_hashmap;
mod distributing_queue;
mod identifiable_object;
mod shape;
mod aabb_iterator;
mod hash_coordinates;
mod shrink_storage;
mod unique_id_allocator;
mod unique_object_storage;
use shape::*;
use spatial_hashmap::*;
use std::sync::*;
use identifiable_object::*;
use unique_id_allocator::*;
use unique_object_storage::*;

struct DynamicObject {
    id: ReturnableId,
    position: Shape,
    spatial_map: Arc<spatial_hashmap::SpatialHashmap>,
    storage: Arc<UniqueObjectStorage>
}

impl StoredObject for DynamicObject {
    fn get_id(&self) -> IdType {
        return self.id.id;
    }

    fn tick(self: Arc<Self>) {
        self.spatial_map.add(self.clone());
    }
}

impl CollidableObject for DynamicObject {

    fn get_shape(&self) -> &Shape {
        return &self.position;
    }

    fn get_id(&self) -> IdType {
        return self.id.id;
    }

    fn collide_with(&self, shape: &Shape, from: IdType) {
        //println!("From ID: {}, Shape type: {:?}", from, shape);
    }
}

impl DynamicObject {
    pub fn new(spatial_map: Arc<spatial_hashmap::SpatialHashmap>, storage: Arc<UniqueObjectStorage>, position: Shape, id: ReturnableId) -> DynamicObject {
        return DynamicObject {spatial_map: spatial_map, storage: storage, id: id, position: position};
    }
}

struct ViewportObject {
    spatial_map: Arc<spatial_hashmap::SpatialHashmap>,
    position: Shape
}

fn main() {
    let map = Arc::new(spatial_hashmap::SpatialHashmap::new());
    let storage = Arc::new(UniqueObjectStorage::new());
    let unique_id_generator = UniqueIdAllocator::new();

    for x in (-100..100).step_by(2) {
        for y in (-100..100).step_by(2) {
            storage.add(Arc::new(DynamicObject::new(map.clone(), storage.clone(), Shape::Circle(CircleData{x: x as f64 + 0.5, y: y as f64 + 0.5, r: 0.9}), unique_id_generator.new_allocated_id())));
        }
    }

        for _x in 0..10 {
            storage.tick();
            map.run_collisions();
        }
}

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
mod id_type;
mod unique_object;
mod collision_component;
mod shape;
mod aabb_iterator;
mod hash_coordinates;
mod shrink_storage;
mod unique_id_allocator;
mod unique_object_storage;
use shape::*;
use unique_object::*;
use collision_component::*;
use std::sync::*;
use unique_id_allocator::*;
use unique_object_storage::*;
use rayon::prelude::*;
use id_type::*;
use crossbeam_channel::{unbounded, TryRecvError};
use dashmap::DashMap;
use dashmap::DashSet;
use fxhash::FxBuildHasher;
use macroquad::prelude::*;

struct DynamicObject {
    id: ReturnableId,
    collision_component: CollisionComponent
}

impl UniqueObject for DynamicObject {
    fn get_id(&self) -> IdType {
        return self.id.id;
    }

    fn tick(&self) {
        self.collision_component.clear();
    }

    fn as_collision_component(&self) -> Option<&dyn CollidableObject> {
        return Some(self);
    }
}

impl DynamicObject {
    pub fn new(position: Shape, id: ReturnableId) -> DynamicObject {
        return DynamicObject {id: id, collision_component: CollisionComponent::new(position)};
    }
}

impl CollidableObject for DynamicObject {
    fn do_collision(&self, shape: &Shape, id: IdType) {
    }

    fn get_collision_component(&self) -> &CollisionComponent {
        return &self.collision_component;
    }

    fn as_dyn_collidable_object(&self) -> &dyn CollidableObject {
        return self;
    }
}

struct DynamicObjectMessageData {
    pub x: f64,
    pub y: f64,
    pub rotation: f32,
    pub vx: f32,
    pub vy: f32,
    pub angular_velocity: f32,
    pub object_type: IdType,
    pub id: IdType
}

struct DynamicObjectClientData {
    pub x: f64,
    pub y: f64,
    pub rotation: f32,
    pub vx: f32,
    pub vy: f32,
    pub angular_velocity: f32,
    pub object_type: IdType
}

struct DynamicObjectCreationData {
    pub id: IdType
}

struct DynamicObjectDestructionData {
    pub id: IdType
}

enum ServerClientMessage {
    DynamicObjectUpdate(DynamicObjectMessageData),
    DynamicObjectCreation(DynamicObjectCreationData),
    DynamicObjectDestruction(DynamicObjectDestructionData)

}

enum ClientServerMessage {

}

struct InternalViewport {
    id: ReturnableId,
    collision_component: CollisionComponent,
    outgoing_messages: crossbeam_channel::Sender<ServerClientMessage>,
    last_tick_ids: DashSet<IdType, FxBuildHasher>
}

impl InternalViewport {
    fn new(position: Shape, id: ReturnableId, outgoing_queue: crossbeam_channel::Sender<ServerClientMessage>) -> InternalViewport {
        return InternalViewport{id: id, collision_component: CollisionComponent::new(position), outgoing_messages: outgoing_queue, last_tick_ids: DashSet::with_hasher(FxBuildHasher::default())}
    }
}

impl UniqueObject for InternalViewport {
    fn get_id(&self) -> IdType {
        return self.id.id;
    }

    fn tick(&self) {
        let current_tick_list = self.collision_component.get_collision_tracker().get_list();
        let removed: Vec<IdType> = self.last_tick_ids.par_iter().map(|x| *x).filter(|x| !current_tick_list.contains(&x)).collect();

        for remove in removed {
            self.outgoing_messages.send(ServerClientMessage::DynamicObjectDestruction(DynamicObjectDestructionData{id: remove})).unwrap();
        }

        self.last_tick_ids.clear();

        for x in current_tick_list {
            self.last_tick_ids.insert(x);
        }

        shrink_storage!(self.last_tick_ids);

        self.collision_component.clear();
    }

    fn as_collision_component(&self) -> Option<&dyn CollidableObject> {
        return Some(self);
    }
}

impl CollidableObject for InternalViewport {
    fn do_collision(&self, shape: &Shape, id: IdType) {
        let center = shape.center();
        match self.last_tick_ids.contains(&id) {
            true => {
            }, 
            false => {
                self.outgoing_messages.send(ServerClientMessage::DynamicObjectCreation(DynamicObjectCreationData{id: id})).unwrap();
            }
        }

        self.outgoing_messages.send(ServerClientMessage::DynamicObjectUpdate(DynamicObjectMessageData{id: id,
            x: center.0, y: center.1, rotation: 0.0, vx: 0.0, vy: 0.0, angular_velocity: 0.0, object_type: 0})).unwrap();
    }

    fn get_collision_component(&self) -> &CollisionComponent {
        return &self.collision_component;
    }

    fn as_dyn_collidable_object(&self) -> &dyn CollidableObject {
        return self;
    }
}

struct FrontendViewport {
    incoming_messages: crossbeam_channel::Receiver<ServerClientMessage>,
    lag_compensation_cache: DashMap<IdType, DynamicObjectClientData, FxBuildHasher>
}

impl FrontendViewport {
    fn new(incoming_queue: crossbeam_channel::Receiver<ServerClientMessage>) -> FrontendViewport {
        return FrontendViewport{incoming_messages: incoming_queue, lag_compensation_cache: DashMap::with_hasher(FxBuildHasher::default())}
    }

    async fn tick(&self, delta_t: f32) {
        self.lag_compensation_cache.par_iter_mut().for_each(|mut x| {
            let mut value = x.value_mut();
            value.x += (value.vx * delta_t) as f64;
            value.y += (value.vy * delta_t) as f64;
            value.rotation += value.angular_velocity * delta_t;
        });

        loop {
            let message = match self.incoming_messages.try_recv() {
                Ok(has) => has,
                Err(TryRecvError::Empty) => {break;}
                Err(TryRecvError::Disconnected) => {panic!("Server disconnected")}
            };

            match message {
                ServerClientMessage::DynamicObjectCreation(created) => {
                    self.lag_compensation_cache.insert(created.id, DynamicObjectClientData{
                        x: 0.0, y: 0.0, rotation: 0.0, vx: 0.0, vy: 0.0, angular_velocity: 0.0, object_type: 0});
                },
                ServerClientMessage::DynamicObjectDestruction(deleted) => {
                    self.lag_compensation_cache.remove(&deleted.id);
                },
                ServerClientMessage::DynamicObjectUpdate(update) => {
                    match self.lag_compensation_cache.entry(update.id) {
                        dashmap::mapref::entry::Entry::Vacant(_vacant) => (),
                        dashmap::mapref::entry::Entry::Occupied(has) => {
                            has.replace_entry(DynamicObjectClientData{x: update.x, y: update.y, rotation: update.rotation, vx: update.vx, vy: update.vy, angular_velocity: update.angular_velocity, object_type: update.object_type});
                        }
                    }
                }
            }
        }

        self.render().await;
    }

    async fn render(&self) {
        for x in &self.lag_compensation_cache {
            draw_circle((screen_width()/2.0) - x.value().x as f32, (screen_height()/2.0) - x.value().y as f32, 20.0, YELLOW);
        }
        next_frame().await;
    }
}

#[macroquad::main("BasicShapes")]
async fn main() {
    let map = Arc::new(spatial_hashmap::SpatialHashmap::new());
    let storage = Arc::new(UniqueObjectStorage::new());
    let unique_id_generator = UniqueIdAllocator::new();

    let(s, r) = unbounded();

    storage.add(Arc::new(DynamicObject::new(Shape::Circle(CircleData{x: 0.0, y: 0.0, r: 1.0}), unique_id_generator.new_allocated_id())));
    storage.add(Arc::new(InternalViewport::new(Shape::Circle(CircleData{x: 0.0, y: 0.0, r: 10.0}), unique_id_generator.new_allocated_id(), s)));

    let viewport = FrontendViewport::new(r);

    let mut timestamp = std::time::Instant::now();

        loop {
            let objects = storage.all_objects();
            map.run_collisions(objects.as_slice());
            objects.par_iter().for_each(|x| x.tick());
            viewport.tick({
                let new_now = std::time::Instant::now();
                let duration = new_now.duration_since(timestamp).as_secs_f32();
                timestamp = new_now;
                duration}).await;

        }
}

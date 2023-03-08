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

use crate::backend::shape::*;
use crate::backend::unique_object_storage::{unique_object::*, unique_object_storage::*, unique_id_allocator::*};
use crate::backend::world_objects::object_properties::collision_property::*;
use crate::shared_types::*;
use crate::connectivity::server_client_message::*;
use crate::connectivity::dynamic_object_message_data::*;
use dashmap::DashSet;
use tokio::sync::broadcast;
use std::sync::Arc;
use std::sync::*;
use crate::backend::world_interaction_event::*;

pub struct ServerViewport {
    id: ReturnableId,
    shape: Mutex<Shape>,
    already_collided: AlreadyCollidedTracker,
    outgoing_messages: broadcast::Sender<ServerClientMessage>,
    last_tick_ids: DashSet<IdType>,
    unique_object_storage: Arc<UniqueObjectStorage>
}

impl ServerViewport {
    pub fn new(position: Shape, id: ReturnableId, outgoing_queue: broadcast::Sender<ServerClientMessage>, storage: Arc<UniqueObjectStorage>) -> ServerViewport {
        return ServerViewport{id: id, shape: Mutex::new(position), already_collided: AlreadyCollidedTracker::new(), outgoing_messages: outgoing_queue, last_tick_ids: DashSet::new(), unique_object_storage: storage};
    }
}

impl UniqueObject for ServerViewport {
    fn get_id(&self) -> IdType {
        return self.id.id;
    }

    fn get_type(&self) -> ObjectType {
        return ObjectType::Viewport();
    }

    fn tick(&self, _delta_t: DeltaT) -> Vec<WorldInteractionEvent> {
        let current_tick_list = self.already_collided.get_list();
        let removed = self.last_tick_ids.iter().map(|x| *x).filter(|x| !current_tick_list.contains(&x));

        for remove in removed {
            self.outgoing_messages.send(ServerClientMessage::DynamicObjectDestruction(DynamicObjectDestructionData{id: remove})).unwrap();
        }

        self.last_tick_ids.clear();

        for x in current_tick_list {
            self.last_tick_ids.insert(x);
        }

        crate::shrink_storage!(self.last_tick_ids);

        self.already_collided.clear();
        return Vec::new();
    }

    fn get_collision_property(&self) -> Option<&dyn CollidableObject> {
        return Some(self);
    }
}

impl CollidableObject for ServerViewport {
    fn do_collision(&self, _shape: &Shape, id: IdType) {
        let collided_object = match self.unique_object_storage.get_by_id(id) {
            Some(has) => has,
            None => {return;}
        };

        let ship_type =  collided_object.get_type();

        match self.last_tick_ids.contains(&id) {
            true => {
            }, 
            false => {
                self.outgoing_messages.send(ServerClientMessage::DynamicObjectCreation(DynamicObjectCreationData{id: id})).unwrap();
            }
        }

        let coordinates = collided_object.get_collision_property().unwrap().get_shape().center();
        self.outgoing_messages.send(ServerClientMessage::DynamicObjectUpdate(DynamicObjectMessageData{id: id,
            x: coordinates.x,
            y: coordinates.y,
            rotation: 0.0,
            vx: 0.0,
            vy: 0.0,
            angular_velocity: 0.0,
            object_type: ship_type})).unwrap();
    }

    fn get_already_collided(&self) -> &AlreadyCollidedTracker {
        return &self.already_collided;
    }

    fn get_shape(&self) -> Shape {
        return self.shape.lock().unwrap().clone();
    }

}
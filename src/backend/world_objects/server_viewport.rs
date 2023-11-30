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
use crate::backend::shrink_storage::ImmutableShrinkable;
use crate::backend::world_object_storage::{ephemeral_id_allocator::*, world_object::*};
use crate::backend::world_objects::object_properties::collision_property::*;
use crate::connectivity::dynamic_object_message_data::*;
use crate::connectivity::server_client_message::*;
use crate::shared_types::*;
use dashmap::DashSet;
use std::sync::*;
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;

pub struct ServerViewport {
    id: IdType,
    shape: Mutex<Shape>,
    already_collided: AlreadyCollidedTracker,
    outgoing_messages: UnboundedSender<ServerClientMessage>,
    last_tick_ids: DashSet<IdType>,
}

impl ServerViewport {
    pub fn new(
        position: Shape,
        id_generator: IdAllocatorType,
        outgoing_queue: UnboundedSender<ServerClientMessage>,
    ) -> ServerViewport {
        return ServerViewport {
            id: id_generator.new_id(),
            shape: Mutex::new(position),
            already_collided: AlreadyCollidedTracker::new(),
            outgoing_messages: outgoing_queue,
            last_tick_ids: DashSet::new(),
        };
    }
}

impl WorldObject for ServerViewport {
    fn get_id(&self) -> IdType {
        return self.id;
    }

    fn get_type(&self) -> ObjectType {
        return ObjectType::Viewport();
    }

    fn tick(&self, _delta_t: DeltaT) {
        let current_tick_list = self.already_collided.get_list();
        let removed = self
            .last_tick_ids
            .iter()
            .map(|x| *x)
            .filter(|x| !current_tick_list.contains(&x));

        for remove in removed {
            let _ = self.outgoing_messages
                .send(ServerClientMessage::DynamicObjectDestruction(
                    DynamicObjectDestructionData { id: remove },
                )); // Nothing we can do about send errors for users disconnected
        }

        self.last_tick_ids.clear();

        for x in current_tick_list.iter() {
            self.last_tick_ids.insert(*x);
        }

        self.last_tick_ids.shrink_storage();

        self.already_collided.clear();
    }

    fn get_collision_property(&self) -> Option<&dyn CollidableObject> {
        return Some(self);
    }

    fn get_serialization_data(&self) -> WorldObjectSerializationData {
        WorldObjectSerializationData::None
    }
}

impl CollidableObject for ServerViewport {
    fn do_collision(&self, collided_object: &dyn WorldObject) {
        let id = collided_object.get_id();
        let ship_type = collided_object.get_type();

        match self.last_tick_ids.contains(&id) {
            true => {}
            false => {
                let _ = self.outgoing_messages
                    .send(ServerClientMessage::DynamicObjectCreation(
                        DynamicObjectCreationData { id: id },
                    )); // Nothing we can do about send errors for users disconnected
            }
        }

        let coordinates = collided_object
            .get_collision_property()
            .unwrap()
            .get_shape()
            .center();

        let _ = self.outgoing_messages
            .send(ServerClientMessage::DynamicObjectUpdate(
                DynamicObjectMessageData {
                    id: id,
                    x: coordinates.x,
                    y: coordinates.y,
                    rotation: 0.0,
                    vx: 0.0,
                    vy: 0.0,
                    angular_velocity: 0.0,
                    object_type: ship_type,
                },
            )); // Nothing we can do about send errors for users disconnected

    }

    fn get_already_collided(&self) -> &AlreadyCollidedTracker {
        return &self.already_collided;
    }

    fn get_shape(&self) -> Shape {
        return self.shape.lock().unwrap().clone();
    }

    fn set_shape(&self, shape: Shape) -> Shape {
        let mut locked = self.shape.lock().unwrap();
        let old_shape = *locked;
        *locked = shape;
        return old_shape;

    }
}

use super::shape::*;
use super::unique_object::*;
use super::collision_component::*;
use super::unique_id_allocator::*;
use super::super::shared_types::*;
use super::super::connectivity::server_client_message::*;
use super::super::connectivity::dynamic_object_message_data::*;
use dashmap::DashSet;
use fxhash::FxBuildHasher;
use std::sync::Arc;
use super::unique_object_storage::*;
use std::sync::*;
use super::world_interaction_event::*;

pub struct ServerViewport {
    id: ReturnableId,
    collision_component: CollisionComponentViewport,
}

impl ServerViewport {
    pub fn new(position: Shape, id: ReturnableId, outgoing_queue: crossbeam_channel::Sender<ServerClientMessage>, storage: Arc<UniqueObjectStorage>) -> ServerViewport {
        return ServerViewport{id: id, collision_component: CollisionComponentViewport::new(position, outgoing_queue, storage)};
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
        self.collision_component.tick();
        return Vec::new();
    }

    fn as_collision_component(&self) -> Option<&dyn CollidableObject> {
        return Some(&self.collision_component);
    }
}

pub struct CollisionComponentViewport {
    shape: Mutex<Shape>,
    already_collided: AlreadyCollidedTracker,
    outgoing_messages: crossbeam_channel::Sender<ServerClientMessage>,
    last_tick_ids: DashSet<IdType, FxBuildHasher>,
    unique_object_storage: Arc<UniqueObjectStorage>
}

impl CollisionComponentViewport {
    pub fn new(shape: Shape,
        outgoing_messages: crossbeam_channel::Sender<ServerClientMessage>,
        unique_object_storage: Arc<UniqueObjectStorage>) -> CollisionComponentViewport {
        return CollisionComponentViewport{
            shape: Mutex::new(shape),
            already_collided: AlreadyCollidedTracker::new(),
            last_tick_ids: DashSet::with_hasher(FxBuildHasher::default()),
            outgoing_messages: outgoing_messages,
            unique_object_storage: unique_object_storage};
    }

    pub fn tick(&self) {
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

        self.clear();
    }
}

impl CollidableObject for CollisionComponentViewport {
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

        match collided_object.as_motion_component() {
            Some(motion) => {
                let coordinates = motion.get_coordinates();
                self.outgoing_messages.send(ServerClientMessage::DynamicObjectUpdate(DynamicObjectMessageData{id: id,
                    x: coordinates.location.x,
                    y: coordinates.location.y,
                    rotation: coordinates.rotation.get(),
                    vx: coordinates.velocity.x,
                    vy: coordinates.velocity.y,
                    angular_velocity: coordinates.angular_velocity.get(),
                    object_type: ship_type})).unwrap();
            },
            None => {
                match collided_object.as_collision_component() {
                    Some(collision) => {
                        let coordinates = collision.get_shape().center();
                        self.outgoing_messages.send(ServerClientMessage::DynamicObjectUpdate(DynamicObjectMessageData{id: id,
                            x: coordinates.x,
                            y: coordinates.y,
                            rotation: 0.0,
                            vx: 0.0,
                            vy: 0.0,
                            angular_velocity: 0.0,
                            object_type: ship_type})).unwrap();
                    },
                    None => ()
                }
            }
        }
    }

    fn get_already_collided(&self) -> &AlreadyCollidedTracker {
        return &self.already_collided;
    }

    fn get_shape(&self) -> Shape {
        return self.shape.lock().unwrap().clone();
    }

    fn set_shape(&self, new_shape: Shape) {
        *self.shape.lock().unwrap() = new_shape;
    }
}
use super::shape::*;
use super::unique_object::*;
use super::collision_component::*;
use super::unique_id_allocator::*;
use super::super::shared_types::*;
use super::super::connectivity::server_client_message::*;
use super::super::connectivity::dynamic_object_message_data::*;
use rayon::prelude::*;
use dashmap::DashSet;
use fxhash::FxBuildHasher;
use macroquad::prelude::*;
use std::sync::Arc;
use super::unique_object_storage::*;

pub struct ServerViewport {
    id: ReturnableId,
    collision_component: CollisionComponent,
    outgoing_messages: crossbeam_channel::Sender<ServerClientMessage>,
    last_tick_ids: DashSet<IdType, FxBuildHasher>,
    unique_object_storage: Arc<UniqueObjectStorage>
}

impl ServerViewport {
    pub fn new(position: Shape, id: ReturnableId, outgoing_queue: crossbeam_channel::Sender<ServerClientMessage>, storage: Arc<UniqueObjectStorage>) -> ServerViewport {
        return ServerViewport{id: id,
            collision_component: CollisionComponent::new(position),
            outgoing_messages: outgoing_queue,
            last_tick_ids: DashSet::with_hasher(FxBuildHasher::default()),
            unique_object_storage: storage}
    }
}

impl UniqueObject for ServerViewport {
    fn get_id(&self) -> IdType {
        return self.id.id;
    }

    fn get_type(&self) -> ObjectType {
        return ObjectType::NonWorld();
    }

    fn tick(&self, _delta_t: f32) {
        let current_tick_list = self.collision_component.get_collision_tracker().get_list();
        let removed: Vec<IdType> = self.last_tick_ids.par_iter().map(|x| *x).filter(|x| !current_tick_list.contains(&x)).collect();

        for remove in removed {
            self.outgoing_messages.send(ServerClientMessage::DynamicObjectDestruction(DynamicObjectDestructionData{id: remove})).unwrap();
        }

        self.last_tick_ids.clear();

        for x in current_tick_list {
            self.last_tick_ids.insert(x);
        }

        crate::shrink_storage!(self.last_tick_ids);

        self.collision_component.clear();
    }

    fn as_collision_component(&self) -> Option<&dyn CollidableObject> {
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

        match collided_object.as_motion_component() {
            Some(motion) => {
                let coordinates = motion.get_coordinates();
                self.outgoing_messages.send(ServerClientMessage::DynamicObjectUpdate(DynamicObjectMessageData{id: id,
                    x: coordinates.x,
                    y: coordinates.y,
                    rotation: coordinates.r,
                    vx: coordinates.dx,
                    vy: coordinates.dy,
                    angular_velocity: coordinates.dr,
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

    fn get_collision_component(&self) -> &CollisionComponent {
        return &self.collision_component;
    }

    fn as_dyn_collidable_object(&self) -> &dyn CollidableObject {
        return self;
    }
}
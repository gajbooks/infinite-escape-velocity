use std::{
    ops::Add,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
};

use euclid::Scale;
use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::{
    backend::{
        shape::Shape,
        world_object_storage::{
            ephemeral_id_allocator::{EphemeralIdAllocator, IdAllocatorType},
            world_object::{WorldObject, WorldObjectSerializationData},
        },
    },
    shared_types::*,
};

use super::object_properties::collision_property::{AlreadyCollidedTracker, CollidableObject};

#[derive(Serialize, Deserialize)]
pub struct ShipSerializationData {
    shape: Shape,
    prototype: String,
}

pub struct Ship {
    id: IdType,
    shape: Mutex<Shape>,
    prototype: String,
    already_collided: AlreadyCollidedTracker,
    timeout: Mutex<DeltaT>,
    deleted: AtomicBool,
}

impl Ship {
    pub fn new(shape: Shape, prototype: String, id: IdType) -> Ship {
        Self::new_from_serialized(
            &ShipSerializationData {
                shape: shape,
                prototype: prototype,
            },
            id,
        )
    }

    pub fn new_from_serialized(data: &ShipSerializationData, id: IdType) -> Ship {
        Ship {
            id: id,
            shape: Mutex::new(data.shape.clone()),
            already_collided: AlreadyCollidedTracker::new(),
            prototype: data.prototype.clone(),
            timeout: Mutex::new(Scale::new(5.0)),
            deleted: AtomicBool::new(false),
        }
    }
}

impl WorldObject for Ship {
    fn get_id(&self) -> IdType {
        self.id
    }

    fn get_type(&self) -> ObjectType {
        if self.deleted.load(Ordering::Relaxed) {
            ObjectType::Deleted()
        } else {
            crate::shared_types::ObjectType::WorldObject(self.prototype.clone())
        }
    }

    fn get_collision_property(&self) -> Option<&dyn CollidableObject> {
        return Some(self);
    }

    fn get_serialization_data(&self) -> WorldObjectSerializationData {
        WorldObjectSerializationData::Ship(ShipSerializationData {
            shape: self.get_shape(),
            prototype: self.prototype.clone(),
        })
    }

    fn tick(&self, delta_t: crate::shared_types::DeltaT) {
        let new_time;
        {
            let mut timeout = self.timeout.lock().unwrap();
            let old_time = timeout.get();
            new_time = old_time - delta_t.get();
            *timeout = Scale::new(new_time);
        }
        
        if new_time <= 0.0 {
            self.deleted.store(true, Ordering::Relaxed);
        }

        self.already_collided.clear();
    }
}

impl CollidableObject for Ship {
    fn do_collision(&self, collided_object: &dyn WorldObject) {}

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

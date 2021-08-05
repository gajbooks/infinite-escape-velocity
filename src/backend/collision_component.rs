use super::super::shared_types::*;
use super::shape::*;
use std::sync::Mutex;

pub struct AlreadyCollidedTracker {
    list: Mutex<Vec<IdType>>,
}

impl AlreadyCollidedTracker {
    pub fn clear(&self) {
        let mut locked = self.list.lock().unwrap();
        crate::shrink_storage!(locked);
        locked.clear();
    }

    pub fn not_collided(&self, id: IdType) -> bool {
        let mut locked = self.list.lock().unwrap();
        match locked.contains(&id) {
            true => return false,
            false => {
                locked.push(id);
                return true;
            }
        }
    }

    pub fn get_list(&self) -> Vec<IdType> {
        return self.list.lock().unwrap().clone();
    }

    pub fn new() -> AlreadyCollidedTracker {
        AlreadyCollidedTracker {
            list: Mutex::new(Vec::new()),
        }
    }
}

pub struct CollisionComponent {
    shape: Mutex<Shape>,
    already_collided: AlreadyCollidedTracker
}

impl CollisionComponent {
    pub fn new(shape: Shape) -> CollisionComponent {
        return CollisionComponent{shape: Mutex::new(shape), already_collided: AlreadyCollidedTracker::new()};
    }

    pub fn collide_with(&self, target: &dyn CollidableObject, shape: &Shape, id: IdType) {
        if self.already_collided.not_collided(id) {
            target.do_collision(shape, id);
        }
    }

    pub fn get_shape(&self) -> Shape {
        return self.shape.lock().unwrap().clone();
    }

    pub fn set_shape(&self, new_shape: Shape) -> () {
        *self.shape.lock().unwrap() = new_shape;
    }

    pub fn clear(&self) -> () {
        self.already_collided.clear();
    }

    pub fn get_collision_tracker(&self) -> &AlreadyCollidedTracker {
        return &self.already_collided;
    }
}

pub trait CollidableObject {
    fn do_collision(&self, shape: &Shape, id: IdType);
    fn get_collision_component(&self) -> &CollisionComponent;

    fn as_dyn_collidable_object(&self) -> &dyn CollidableObject;

    fn collide_with(&self, shape: &Shape, id: IdType) {
        self.get_collision_component().collide_with(self.as_dyn_collidable_object(), shape, id);
    }

    fn get_shape(&self) -> Shape {
        return self.get_collision_component().get_shape();
    }
}
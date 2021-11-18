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

pub trait CollidableObject {
    fn do_collision(&self, shape: &Shape, id: IdType);

    fn get_already_collided(&self) -> &AlreadyCollidedTracker;

    fn get_shape(&self) -> Shape;

    fn set_shape(&self, new_shape: Shape);

    fn move_center(&self, new_center: Coordinates) {
        self.set_shape(self.get_shape().move_center(new_center));
    }

    fn collide_with(&self, shape: &Shape, id: IdType) {
        if self.get_already_collided().not_collided(id) {
            self.do_collision(shape, id);
        }
    }

    fn clear(&self) -> () {
        self.get_already_collided().clear();
    }
}
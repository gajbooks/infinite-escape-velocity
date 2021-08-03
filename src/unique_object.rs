use super::collision_component::*;
use super::id_type::*;

pub trait UniqueObject {
    fn get_id(&self) -> IdType;
    fn tick(&self);
    fn as_collision_component(&self) -> Option<&dyn CollidableObject> {
        return None;
    }
}
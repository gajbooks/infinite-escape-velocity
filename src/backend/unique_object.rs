use super::collision_component::*;
use super::motion_component::*;
use super::super::shared_types::*;

pub trait UniqueObject {
    fn get_id(&self) -> IdType;
    fn get_type(&self) -> ObjectType;
    fn tick(&self, delta_t: f32);
    fn as_collision_component(&self) -> Option<&dyn CollidableObject> {
        return None;
    }
    fn as_motion_component(&self) -> Option<&dyn MobileObject> {
        return None;
    }
}
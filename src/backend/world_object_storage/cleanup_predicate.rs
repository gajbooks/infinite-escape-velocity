use crate::shared_types::ObjectType;

use super::world_object::WorldObject;


pub fn retain_world_objects(object: &dyn WorldObject) -> bool {
    object.get_type() != ObjectType::Deleted()
}
use super::unique_object::*;
use super::collision_component::*;
use super::motion_component::*;

pub trait DynamicObject: UniqueObject + CollidableObject + MobileObject{}


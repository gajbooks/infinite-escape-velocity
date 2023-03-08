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

use crate::backend::world_interaction_event::WorldInteractionEvent;
use crate::backend::world_objects::object_properties::collision_property::CollidableObject;

use crate::shared_types::*;

pub trait UniqueObject {
    fn get_id(&self) -> IdType;
    fn get_type(&self) -> ObjectType;
    fn get_collision_property(&self) -> Option<&dyn CollidableObject> {
        return None;
    }
    fn tick(&self, _delta_t: DeltaT) -> Vec<WorldInteractionEvent>;
}
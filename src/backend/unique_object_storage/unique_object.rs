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

use super::super::collision_component::*;
use super::super::motion_component::*;
use super::super::controllable_component::*;
use super::super::super::shared_types::*;
use super::super::world_interaction_event::*;

pub trait UniqueObject {
    fn get_id(&self) -> IdType;
    fn get_type(&self) -> ObjectType;
    fn tick(&self, delta_t: DeltaT) -> Vec<WorldInteractionEvent>;
    fn as_collision_component(&self) -> Option<&dyn CollidableObject> {
        return None;
    }
    fn as_motion_component(&self) -> Option<&dyn MobileObject> {
        return None;
    }
    fn as_controllable_component(&self) -> Option<&dyn ControllableObject> {
        return None;
    }
}
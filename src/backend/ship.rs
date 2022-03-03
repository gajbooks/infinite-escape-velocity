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

use super::super::configuration_loaders::object_configuration_record::*;
use super::super::shared_types::*;
use super::collision_component::*;
use super::controllable_component::*;
use super::motion_component::*;
use super::shape::*;
use super::unique_object_storage::{unique_object::*, unique_id_allocator::*};
use super::world_interaction_event::*;
use super::world_object_constructor::*;
use std::sync::*;

pub struct Ship {
    id: ReturnableId,
    collision_component: Arc<CollisionComponentShip>,
    motion_component: Arc<MaximumSpeedMotionComponent>,
    controllable_component: Arc<ControllableComponentShip>,
    object_type: ObjectType,
}

impl FromPrototype for Ship {
    fn from_prototype(
        object_record: &ObjectConfigurationRecord,
        object_type: ObjectType,
        position: CoordinatesRotation,
        id: ReturnableId,
    ) -> Result<Arc<dyn UniqueObject + Send + Sync>, ()> {
        
        let (collision_parameters, movement_parameters) = match &object_record.object {
            ObjectVariant::Ship{collision_parameters, movement_parameters, ..} => {
                (collision_parameters, movement_parameters)
            },
            ObjectVariant::Munition{..} => { return Err(()); },
            ObjectVariant::Planet{..} => { return Err(()); }
        };

        let shape = match &collision_parameters.circle {
            Some(circle) => Shape::Circle(CircleData {
                location: position.location,
                radius: Radius::new(circle.radius.into()),
            }),
            None => match &collision_parameters.rounded_tube {
                Some(tube) => {
                    let distance = Distance::new(tube.length.into());
                    let point_2 =
                        position.location + Offset::from_lengths(distance, Distance::new(0.0));
                    Shape::RoundedTube(RoundedTubeData {
                        point_1: position.location,
                        point_2: point_2,
                        radius: Radius::new(tube.radius.into()),
                    })
                }
                None => {
                    return Err(());
                }
            },
        };

        let collision_component = Arc::new(CollisionComponentShip::new(shape));

        let motion_component = Arc::new(MaximumSpeedMotionComponent::new_from_position(
            &position,
            movement_parameters.maximum_speed,
            movement_parameters.maximum_acceleration,
            movement_parameters.maximum_angular_velocity,
        ));

        let controllable_component = Arc::new(ControllableComponentShip::new(
            movement_parameters.maximum_speed,
            movement_parameters.maximum_acceleration,
            movement_parameters.maximum_angular_velocity,
            motion_component.clone(),
        ));

        return Ok(Arc::new(Ship {
            id: id,
            collision_component: collision_component,
            motion_component: motion_component,
            controllable_component: controllable_component,
            object_type: object_type,
        }));
    }
}

impl UniqueObject for Ship {
    fn get_id(&self) -> IdType {
        return self.id.id;
    }

    fn get_type(&self) -> ObjectType {
        return self.object_type.clone();
    }

    fn tick(&self, delta_t: DeltaT) -> Vec<WorldInteractionEvent> {
        self.collision_component.clear();
        self.motion_component.apply_velocity_tick(delta_t);

        let mut events = Vec::new();

        self.controllable_component.tick();

        let updated_pos = self.motion_component.get_coordinates();
        self.collision_component.move_center(updated_pos.location);

        return events;
    }

    fn as_collision_component(&self) -> Option<&dyn CollidableObject> {
        return Some(&*self.collision_component);
    }

    fn as_motion_component(&self) -> Option<&dyn MobileObject> {
        return Some(&*self.motion_component);
    }

    fn as_controllable_component(&self) -> Option<&dyn ControllableObject> {
        return Some(&*self.controllable_component);
    }
}

pub struct CollisionComponentShip {
    shape: Mutex<Shape>,
    already_collided: AlreadyCollidedTracker,
}

impl CollisionComponentShip {
    pub fn new(shape: Shape) -> CollisionComponentShip {
        return CollisionComponentShip {
            shape: Mutex::new(shape),
            already_collided: AlreadyCollidedTracker::new(),
        };
    }
}

impl CollidableObject for CollisionComponentShip {
    fn do_collision(&self, shape: &Shape, id: IdType) {}

    fn get_already_collided(&self) -> &AlreadyCollidedTracker {
        return &self.already_collided;
    }

    fn get_shape(&self) -> Shape {
        return self.shape.lock().unwrap().clone();
    }

    fn set_shape(&self, new_shape: Shape) {
        *self.shape.lock().unwrap() = new_shape;
    }
}

use super::shape::*;
use super::unique_object::*;
use super::collision_component::*;
use super::motion_component::*;
use super::unique_id_allocator::*;
use super::super::shared_types::*;
use super::controllable_component::*;
use std::sync::*;
use super::world_object_constructor::*;
use super::super::configuration_loaders::dynamic_object_record::*;
use super::world_interaction_event::*;

pub struct Ship {
    id: ReturnableId,
    collision_component: Arc<CollisionComponentShip>,
    motion_component: Arc<MaximumSpeedMotionComponent>,
    controllable_component: Arc<ControllableComponentShip>,
    object_type: ObjectType
}

impl FromPrototype for Ship {
    fn from_prototype(object_record: &DynamicObjectRecord, object_type: ObjectType, position: CoordinatesRotation, id: ReturnableId) -> Result<Arc<dyn UniqueObject + Send + Sync>, ()> {
        let movement_parameters = match &object_record.movement_parameters {
            Some(movement) => movement,
            None => {
                return Err(());
            }
        };

        let collision_parameters = match &object_record.collision_parameters {
            Some(collision) => {
                collision
            },
            None => {
                return Err(());
            }
        };

        let shape = match &collision_parameters.circle {
            Some(circle) => {
                Shape::Circle(
                    CircleData{
                        location: position.location,
                        radius: Radius::new(circle.radius.into())
                    }
                )
            },
            None => {
                match &collision_parameters.rounded_tube {
                    Some(tube) => {
                        let distance = Distance::new(tube.length.into());
                        let point_2 = position.location + Offset::from_lengths(distance, Distance::new(0.0));
                        Shape::RoundedTube(
                            RoundedTubeData{
                                point_1: position.location,
                                point_2: point_2,
                                radius: Radius::new(tube.radius.into())
                            }
                        )
                    },
                    None => {
                        return Err(());
                    }
                }
            }
        };

        let collision_component = Arc::new(
            CollisionComponentShip::new(shape)
        );

        let motion_component = Arc::new(MaximumSpeedMotionComponent::new_from_position(
            &position,
            movement_parameters.maximum_speed,
            movement_parameters.maximum_acceleration,
            movement_parameters.maximum_angular_velocity));

        let controllable_component = Arc::new(ControllableComponentShip::new(
            movement_parameters.maximum_speed,
            movement_parameters.maximum_acceleration,
            movement_parameters.maximum_angular_velocity,
            motion_component.clone()));

        return Ok(Arc::new(Ship {
            id: id,
            collision_component: collision_component,
            motion_component: motion_component,
            controllable_component: controllable_component,
            object_type: object_type
        }))
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

        self.controllable_component.tick();

        let updated_pos = self.motion_component.get_coordinates();
        self.collision_component.move_center(updated_pos.location);

        let events = Vec::new();

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
    already_collided: AlreadyCollidedTracker
}

impl CollisionComponentShip {
    pub fn new(shape: Shape) -> CollisionComponentShip {
        return CollisionComponentShip{shape: Mutex::new(shape), already_collided: AlreadyCollidedTracker::new()};
    }
}

impl CollidableObject for CollisionComponentShip {

    fn do_collision(&self, shape: &Shape, id: IdType) {

    }

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
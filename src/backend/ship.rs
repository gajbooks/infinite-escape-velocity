use super::shape::*;
use super::unique_object::*;
use super::collision_component::*;
use super::motion_component::*;
use super::unique_id_allocator::*;
use super::super::shared_types::*;
use super::controllable_component::*;
use std::sync::*;

pub struct Ship {
    id: ReturnableId,
    collision_component: Arc<CollisionComponentShip>,
    motion_component: Arc<MaximumSpeedMotionComponent>,
    controllable_component: Arc<ControllableComponentShip>
}

impl Ship {
    pub fn new(position: CoordinatesRotation, id: ReturnableId) -> Ship {
        let collision_component = Arc::new(CollisionComponentShip::new(Shape::Circle(CircleData{location: position.location, radius: Radius::new(1.0)})));
        let motion_component = Arc::new(MaximumSpeedMotionComponent::new_from_position(&position, 100.0, 50.0, 1.0));
        let controllable_component = Arc::new(ControllableComponentShip::new(10000.0, 100.0, 2.0, motion_component.clone()));
        return Ship {
            id: id,
            collision_component: collision_component,
            motion_component: motion_component,
            controllable_component: controllable_component
        }
    }
}

impl UniqueObject for Ship {
    fn get_id(&self) -> IdType {
        return self.id.id;
    }

    fn get_type(&self) -> ObjectType {
        return ObjectType::Ship(ShipTypeData{namespace: 0, id: 0});
    }

    fn tick(&self, delta_t: DeltaT) {
        self.collision_component.clear();
        self.motion_component.apply_velocity_tick(delta_t);
        self.controllable_component.tick();

        let updated_pos = self.motion_component.get_coordinates();
        self.collision_component.move_center(updated_pos.location);
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
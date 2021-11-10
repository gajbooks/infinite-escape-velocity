use super::shape::*;
use super::unique_object::*;
use super::dynamic_object::*;
use super::collision_component::*;
use super::motion_component::*;
use super::unique_id_allocator::*;
use super::super::shared_types::*;

pub struct Ship {
    id: ReturnableId,
    collision_component: CollisionComponent,
    motion_component: MaximumSpeedMotionComponent
}

impl Ship {
    pub fn new(position: &CoordinatesRotation, id: ReturnableId) -> Ship {
        return Ship {id: id, collision_component: CollisionComponent::new(Shape::Circle(CircleData{x: position.x, y: position.y, r: 1.0})), motion_component: MaximumSpeedMotionComponent::new_from_position(&position, 100.0, 90.0)};
    }
}

impl DynamicObject for Ship{}

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
        let updated_pos = self.motion_component.get_coordinates();
        self.collision_component.set_shape(Shape::Circle(CircleData{x: updated_pos.x, y: updated_pos.y, r: 1.0}));
    }

    fn as_collision_component(&self) -> Option<&dyn CollidableObject> {
        return Some(self);
    }

    fn as_motion_component(&self) -> Option<&dyn MobileObject> {
        return Some(self);
    }
}

impl CollidableObject for Ship {
    fn do_collision(&self, shape: &Shape, id: IdType) {
    }

    fn get_collision_component(&self) -> &CollisionComponent {
        return &self.collision_component;
    }

    fn as_dyn_collidable_object(&self) -> &dyn CollidableObject {
        return self;
    }
}

impl MobileObject for Ship {
    fn get_motion_component(&self) -> &dyn MotionComponent {
        return &self.motion_component;
    }
}
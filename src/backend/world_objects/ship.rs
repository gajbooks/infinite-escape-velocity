use bevy_ecs::bundle::Bundle;

use super::{server_viewport::Displayable, object_properties::{collision_component::CollisionMarker, timeout_component::TimeoutComponent}};

#[derive(Bundle)]
pub struct ShipBundle {
    pub displayable: Displayable,
    pub displayable_collision_marker: CollisionMarker<Displayable>,
    pub timeout: TimeoutComponent
}

use std::time::Duration;

use bevy_ecs::{system::{Query, Commands, Res}, component::Component, entity::Entity};

use crate::backend::resources::delta_t_resource::DeltaTResource;


#[derive(Component)]
pub struct TimeoutComponent {
    pub spawn_time: Duration,
    pub lifetime: Duration
}

pub fn check_despawn_times(timeouts: Query<(Entity, &TimeoutComponent)>, time: Res<DeltaTResource>, mut commands: Commands) {
    timeouts.for_each(|(entity, timeout)| {
        if time.total_time - timeout.spawn_time > timeout.lifetime {
            commands.entity(entity).despawn();
        }
    });
}
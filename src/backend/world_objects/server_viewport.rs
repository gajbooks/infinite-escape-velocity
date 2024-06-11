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

use crate::backend::shrink_storage::ImmutableShrinkable;
use crate::backend::spatial_optimizer::hash_sized::HashSized;
use crate::backend::world_objects::components::collision_component::*;
use crate::configuration_file_structures::reference_types::AssetIndexReference;
use crate::connectivity::controllable_object_message_data::ViewportFollowData;
use crate::connectivity::dynamic_object_message_data::*;
use crate::connectivity::server_client_message::*;
use crate::connectivity::user_session::UserSession;
use crate::shared_types::Coordinates;
use bevy_ecs::prelude::*;
use dashmap::DashSet;
use tokio::sync::mpsc::UnboundedSender;
use tracing::warn;

use super::components::angular_velocity_component::AngularVelocityComponent;
use super::components::position_component::PositionComponent;
use super::components::rotation_component::RotationComponent;
use super::components::velocity_component::VelocityComponent;

#[derive(Bundle)]
pub struct ViewportBundle {
    pub viewport: ServerViewport,
    pub collidable: CollidableComponent<Displayable>,
}

#[derive(Component)]
pub struct Displayable {
    pub object_asset: AssetIndexReference,
}

impl HashSized for Displayable {}

pub enum ViewportTrackingMode {
    Entity(Entity),
    Static(Coordinates),
    Disconnected,
}

#[derive(Component)]
pub struct ServerViewport {
    player: Entity,
    last_tick_ids: DashSet<Entity>,
    outgoing_messages: UnboundedSender<ServerClientMessage>,
    tracking_mode: ViewportTrackingMode,
}

impl ServerViewport {
    pub fn new(
        player: Entity,
        outgoing_queue: UnboundedSender<ServerClientMessage>,
    ) -> ServerViewport {
        return ServerViewport {
            player,
            last_tick_ids: DashSet::new(),
            outgoing_messages: outgoing_queue,
            tracking_mode: ViewportTrackingMode::Static(Coordinates::new(0.0, 0.0)),
        };
    }

    pub fn set_tracking_mode(&mut self, new_tracking_mode: ViewportTrackingMode) {
        self.tracking_mode = new_tracking_mode;
        let tracking_message_data = match self.tracking_mode {
            ViewportTrackingMode::Entity(entity) => ViewportFollowData::Entity {
                id: entity.to_bits(),
            },
            ViewportTrackingMode::Static(location) => ViewportFollowData::Static {
                x: location.x,
                y: location.y,
            },
            ViewportTrackingMode::Disconnected => ViewportFollowData::Disconnected,
        };

        let _ = self
            .outgoing_messages
            .send(ServerClientMessage::ViewportFollow(tracking_message_data)); // Nothing we can do about send errors for users disconnected
    }
}

pub fn tick_viewport(
    mut all_viewports: Query<(
        Entity,
        &mut ServerViewport,
        &mut CollidableComponent<Displayable>,
    )>,
    displayables: Query<(
        &CollisionMarker<Displayable>,
        &PositionComponent,
        &Displayable,
    )>,
    optional_velocity: Query<&VelocityComponent>,
    optional_rotation: Query<&RotationComponent>,
    optional_angular_velocity: Query<&AngularVelocityComponent>,
    sessions: Query<&UserSession>,
    mut commands: Commands,
) {
    for (viewport_entity, mut viewport, mut collide_with) in all_viewports.iter_mut() {
        // Despawn the viewport if the owning user session no longer exists
        if sessions.contains(viewport.player) == false {
            commands.entity(viewport_entity).despawn();
            continue;
        }

        match viewport.tracking_mode {
            ViewportTrackingMode::Entity(entity) => {
                // Track viewport to assigned entity
                match displayables.get(entity) {
                    Ok((_, position, _)) => {
                        collide_with.shape = collide_with.shape.move_center(position.position);
                    }
                    Err(_lost_track) => {
                        viewport.tracking_mode = ViewportTrackingMode::Disconnected;
                    }
                }
            }
            ViewportTrackingMode::Static(location) => {
                // Presumably some external force may want to move the viewport to a fixed position unrelated to any entity
                collide_with.shape = collide_with.shape.move_center(location);
            }
            ViewportTrackingMode::Disconnected => {
                // Potentially do something if the viewport has lost contact with its assigned entity
            }
        };

        for collision in collide_with.list.iter().map(|x| x.key().clone()) {
            // Theoretically we could get an entity in the collision list that doesn't match the query, we should just ignore them
            let (_collided_hitbox, position, displayable) = match displayables.get(collision) {
                Ok(x) => x,
                Err(_) => {
                    warn!("Entity collided with Viewport which does not have the required components: {:?}", collision);
                    continue;
                }
            };

            // Send a creation frame for each object not previously within the viewport's range
            match viewport.last_tick_ids.contains(&collision) {
                true => {}
                false => {
                    let _ = viewport.outgoing_messages.send(
                        ServerClientMessage::DynamicObjectCreation(DynamicObjectCreationData {
                            id: collision.to_bits(),
                        }),
                    ); // Nothing we can do about send errors for users disconnected
                }
            }

            let velocity = match optional_velocity.get(collision) {
                Ok(has) => Some(VelocityMessage {
                    vx: has.velocity.x,
                    vy: has.velocity.y,
                }),
                Err(_) => None,
            };

            let rotation = match optional_rotation.get(collision) {
                Ok(has) => Some(RotationMessage {
                    rotation: has.rotation.get(),
                }),
                Err(_) => None,
            };

            let angular_velocity = match optional_angular_velocity.get(collision) {
                Ok(has) => Some(AngularVelocityMessage {
                    angular_velocity: has.angular_velocity.get(),
                }),
                Err(_) => None,
            };

            // Send an update frame for each object moved which has been within the viewport for at least one frame
            let _ = viewport
                .outgoing_messages
                .send(ServerClientMessage::DynamicObjectUpdate(
                    DynamicObjectMessageData {
                        id: collision.to_bits(),
                        x: position.position.x,
                        y: position.position.y,
                        rotation: rotation,
                        velocity: velocity,
                        angular_velocity: angular_velocity,
                        object_asset: displayable.object_asset,
                    },
                )); // Nothing we can do about send errors for users disconnected
        }

        // Find all entities in the previous frame which no longer are within the viewport for the current frame
        let removed = viewport
            .last_tick_ids
            .iter()
            .map(|x| *x)
            .filter(|x| !collide_with.list.contains(&x));

        // Send a destruction frame for all removed entities to guarantee no stale entities remain on the client
        for remove in removed {
            let _ = viewport
                .outgoing_messages
                .send(ServerClientMessage::DynamicObjectDestruction(
                    DynamicObjectDestructionData {
                        id: remove.to_bits(),
                    },
                )); // Nothing we can do about send errors for users disconnected
        }

        // Clear the entities we consider for the last tick
        viewport.last_tick_ids.clear();

        // Add all entities for this tick to the last tick counter
        for x in collide_with.list.iter() {
            viewport.last_tick_ids.insert(*x);
        }

        // Make sure the viewport last tick storage doesn't have a huge amount of excess capacity
        viewport.last_tick_ids.shrink_storage();
    }
}

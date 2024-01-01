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

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::backend::shrink_storage::ImmutableShrinkable;
use crate::backend::world_objects::object_properties::collision_component::*;
use crate::connectivity::dynamic_object_message_data::*;
use crate::connectivity::server_client_message::*;
use bevy_ecs::prelude::*;
use dashmap::DashSet;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Bundle)]
pub struct ViewportBundle {
    pub viewport: ServerViewport,
    pub collidable: CollidableComponent<Displayable>,
}

#[derive(Component)]
pub struct Displayable {
    pub object_type: String,
}

#[derive(Component)]
pub struct ServerViewport {
    cancel: Arc<AtomicBool>,
    outgoing_messages: UnboundedSender<ServerClientMessage>,
    last_tick_ids: DashSet<Entity>,
}

impl ServerViewport {
    pub fn new(
        cancel: Arc<AtomicBool>,
        outgoing_queue: UnboundedSender<ServerClientMessage>,
    ) -> ServerViewport {
        return ServerViewport {
            cancel: cancel,
            outgoing_messages: outgoing_queue,
            last_tick_ids: DashSet::new(),
        };
    }
}

pub fn tick_viewport(
    mut all_viewports: Query<(
        Entity,
        &mut ServerViewport,
        &CollidableComponent<Displayable>,
    )>,
    displayables: Query<(&CollisionMarker<Displayable>, &Displayable)>,
    mut commands: Commands,
) {
    for (viewport_entity, viewport, collide_with) in all_viewports.iter_mut() {
        if viewport.cancel.load(std::sync::atomic::Ordering::Relaxed) == true {
            commands.entity(viewport_entity).despawn();
            continue;
        }

        for collision in collide_with.list.iter().map(|x| x.key().clone()) {
            let (collided_hitbox, displayable) = match displayables.get(collision) {
                Ok(x) => x,
                Err(_) => continue,
            };

            match viewport.last_tick_ids.contains(&collision) {
                true => {}
                false => {
                    let _ = viewport.outgoing_messages.send(
                        ServerClientMessage::DynamicObjectCreation(DynamicObjectCreationData {
                            id: Into::<ExternalEntity>::into(collision),
                        }),
                    ); // Nothing we can do about send errors for users disconnected
                }
            }

            let coordinates = collided_hitbox.shape.center();

            let _ = viewport
                .outgoing_messages
                .send(ServerClientMessage::DynamicObjectUpdate(
                    DynamicObjectMessageData {
                        id: Into::<ExternalEntity>::into(collision),
                        x: coordinates.x,
                        y: coordinates.y,
                        rotation: 0.0,
                        vx: 0.0,
                        vy: 0.0,
                        angular_velocity: 0.0,
                        object_type: displayable.object_type.clone(),
                    },
                )); // Nothing we can do about send errors for users disconnected
        }

        let removed = viewport
            .last_tick_ids
            .iter()
            .map(|x| *x)
            .filter(|x| !collide_with.list.contains(&x));

        for remove in removed {
            let _ = viewport
                .outgoing_messages
                .send(ServerClientMessage::DynamicObjectDestruction(
                    DynamicObjectDestructionData { id: remove.into() },
                )); // Nothing we can do about send errors for users disconnected
        }

        viewport.last_tick_ids.clear();

        for x in collide_with.list.iter() {
            viewport.last_tick_ids.insert(*x);
        }

        viewport.last_tick_ids.shrink_storage();
    }
}

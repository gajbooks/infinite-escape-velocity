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

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, Res},
};

use crate::{
    backend::{
        shape::{CircleData, Shape},
        world_objects::{
            components::{
                collision_component::CollidableComponent,
                player_controlled_component::PlayerControlledComponent,
                semi_newtonian_physics_component::SemiNewtonianPhysicsComponent,
            },
            server_viewport::{ServerViewport, ViewportBundle, ViewportTrackingMode},
            ship::ShipBundle,
        },
    },
    connectivity::user_session::UserSession,
    shared_types::{Coordinates, Radius, Speed},
    AssetIndexResource,
};

pub fn spawn_player_ship_and_viewports(
    mut sessions: Query<(Entity, &mut UserSession)>,
    mut viewports: Query<&mut ServerViewport>,
    mut commands: Commands,
    asset_index: Res<AssetIndexResource>,
) {
    sessions
        .iter_mut()
        .for_each(|(session_entity, mut session)| {
            let following_id = match session.should_follow {
                Some(following) => following,
                None => {
                    let new_ship = ShipBundle::new(
                        Coordinates::new(0.0, 0.0),
                        None,
                        None,
                        None,
                        &asset_index.asset_index,
                    )
                    .unwrap();
                    let new_ship_id = commands
                        .spawn((
                            new_ship,
                            SemiNewtonianPhysicsComponent::new(Speed::new(200.0)),
                            PlayerControlledComponent::new(
                                session.control_input_sender.subscribe(),
                                session.cancel.clone(),
                            ),
                        ))
                        .id();
                    session.should_follow = Some(new_ship_id);
                    new_ship_id
                }
            };

            match session.primary_viewport {
                Some(viewport_exists) => {
                    match viewports.get_mut(viewport_exists) {
                        // Viewport exists already
                        Ok(mut has) => {
                            has.set_tracking_mode(ViewportTrackingMode::Entity(following_id));
                        }
                        // Viewport has somehow been destroyed, forget reference
                        Err(_destroyed) => {
                            session.primary_viewport = None;
                        }
                    }
                }
                None => {
                    let new_viewport = commands
                        .spawn(ViewportBundle {
                            viewport: ServerViewport::new(
                                session_entity,
                                session.to_remote.clone(),
                            ),
                            collidable: CollidableComponent::new(Shape::Circle(CircleData {
                                location: Coordinates::new(0.0, 0.0),
                                radius: Radius::new(6000.0),
                            })),
                        })
                        .id();
                    session.primary_viewport = Some(new_viewport);
                }
            }
        });
}

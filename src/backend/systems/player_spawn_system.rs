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
    hierarchy::ChildOf,
    prelude::{Commands, Query, Res},
};

use crate::{
    AssetIndexResource,
    backend::{
        components::session::player_session_component::PlayerSessionComponent,
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
    shared_types::{Coordinates, Radius, Speed},
};

pub fn spawn_player_ship_and_viewports(
    entities: Query<Entity>,
    mut sessions: Query<(Entity, &mut PlayerSessionComponent)>,
    mut viewports: Query<&mut ServerViewport>,
    mut commands: Commands,
    asset_index: Res<AssetIndexResource>,
) {
    sessions
        .iter_mut()
        .for_each(|(session_entity, mut session)| {
            let following_id = if let Some(following) = session.should_follow {
                if entities.contains(following) {
                    Some(following)
                } else {
                    None
                }
            } else {
                None
            };

            let following_id = match following_id {
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
                            PlayerControlledComponent {},
                            ChildOf(session_entity),
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
                        Ok(has) => {
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
                            viewport: ServerViewport::new(),
                            collidable: CollidableComponent::new(Shape::Circle(CircleData {
                                location: Coordinates::new(0.0, 0.0),
                                radius: Radius::new(6000.0),
                            })),
                            parent_session: ChildOf(session_entity),
                        })
                        .id();
                    session.primary_viewport = Some(new_viewport);
                }
            }
        });
}

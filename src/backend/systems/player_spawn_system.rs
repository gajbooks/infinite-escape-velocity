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

use bevy_ecs::{system::{Query, Commands}, entity::Entity};

use crate::{connectivity::user_session::UserSession, backend::{world_objects::{ship::ShipBundle, server_viewport::{Displayable, ServerViewport, ViewportTrackingMode, ViewportBundle}, object_properties::collision_component::{CollisionMarker, CollidableComponent}}, shape::{PointData, Shape, CircleData}}, shared_types::{Coordinates, Radius}};

pub fn spawn_player_ship_and_viewports(mut sessions: Query<(Entity, &mut UserSession)>, mut viewports: Query<&mut ServerViewport>, mut commands: Commands) {
    sessions.for_each_mut(|(session_entity, mut session)| {
        let following_id = match session.should_follow {
            Some(following) => following,
            None => {
                let new_ship = ShipBundle{
                    displayable: Displayable{object_type: format!("Player Ship {}", session.remote_address)},
                    displayable_collision_marker: CollisionMarker::<Displayable>::new(Shape::Point(PointData{point: Coordinates::new(0.0, 0.0)}))
                };
                let new_ship_id = commands.spawn(new_ship).id();
                session.should_follow = Some(new_ship_id);
                new_ship_id
            },
        };

        match session.primary_viewport {
            Some(viewport_exists) => {
                match viewports.get_mut(viewport_exists) {
                    // Viewport exists already
                    Ok(mut has) => {
                        has.set_tracking_mode(ViewportTrackingMode::Entity(following_id));
                    },
                    // Viewport has somehow been destroyed, forget reference
                    Err(_destroyed) => {
                        session.primary_viewport = None;
                    },
                }
            },
            None => {
                let new_viewport = commands.spawn(ViewportBundle {
                    viewport: ServerViewport::new(session_entity, session.to_remote.clone()),
                    collidable: CollidableComponent::new(Shape::Circle(CircleData{ location: Coordinates::new(0.0, 0.0), radius: Radius::new(2000.0) })),
                }).id();
                session.primary_viewport = Some(new_viewport);
            },
        }
    });
}
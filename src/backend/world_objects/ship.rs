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

use bevy_ecs::bundle::Bundle;

use crate::{
    backend::shape::{PointData, Shape},
    connectivity::asset_index::AssetIndex,
    shared_types::{AngularVelocity, Coordinates, Rotation, Velocity},
};

use super::{
    components::{
        angular_velocity_component::AngularVelocityComponent, collision_component::CollisionMarker,
        position_component::PositionComponent, rotation_component::RotationComponent,
        velocity_component::VelocityComponent,
    },
    server_viewport::Displayable,
};

#[derive(Bundle)]
pub struct ShipBundle {
    pub displayable: Displayable,
    pub displayable_collision_marker: CollisionMarker<Displayable>,
    pub position: PositionComponent,
    pub velocity: VelocityComponent,
    pub rotation: RotationComponent,
    pub angular_velocity: AngularVelocityComponent,
}

impl ShipBundle {
    pub fn new(
        position: Coordinates,
        velocity: Option<Velocity>,
        rotation: Option<Rotation>,
        angular_velocity: Option<AngularVelocity>,
        asset_index: &AssetIndex,
    ) -> Result<Self, ()> {
        // Ship records do not exist yet in this build so just use a fixed asset name for now
        let asset_name = "default_asset";
        let display_asset = match asset_index.lookup_asset_by_name(asset_name) {
            Some(has) => *has,
            None => {
                tracing::warn!(
                    "Attempted to create planetoid with asset index {} which has an id which does not exist",
                    asset_name
                );
                return Err(());
            }
        };

        Ok(Self {
            displayable: Displayable {
                object_asset: display_asset,
            },
            displayable_collision_marker: CollisionMarker::<Displayable>::new(Shape::Point(
                PointData { point: position },
            )),
            position: PositionComponent { position: position },
            velocity: VelocityComponent {
                velocity: velocity.unwrap_or_default(),
            },
            rotation: RotationComponent {
                rotation: rotation.unwrap_or_default(),
            },
            angular_velocity: AngularVelocityComponent {
                angular_velocity: angular_velocity.unwrap_or_default(),
            },
        })
    }
}

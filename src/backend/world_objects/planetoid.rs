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
use euclid::Point2D;

use crate::{
    backend::shape::{CircleData, Shape},
    configuration_file_structures::planetoid_configuration_file::PlanetoidRecord,
    connectivity::{asset_index::AssetIndex, view_layers::ViewLayers},
    shared_types::Radius,
};

use super::{
    components::{collision_component::CollisionMarker, position_component::PositionComponent},
    server_viewport::Displayable,
};

#[derive(Bundle)]
pub struct PlanetoidBundle {
    pub displayable: Displayable,
    pub displayable_collision_marker: CollisionMarker<Displayable>,
    pub position: PositionComponent,
}

impl PlanetoidBundle {
    pub fn new(record: &PlanetoidRecord, asset_index: &AssetIndex) -> Result<Self, ()> {
        let display_asset = match asset_index.lookup_asset_by_name(&record.display_asset) {
            Some(has) => *has,
            None => {
                tracing::warn!(
                    "Attempted to create planetoid with asset index {} which has an id which does not exist",
                    record.display_asset
                );
                return Err(());
            }
        };

        let position = Point2D::new(record.x, record.y);
        let radius = Radius::new(record.display_radius as f64);

        Ok(Self {
            displayable: Displayable {
                display_radius: radius.0 as f32,
                object_asset: display_asset,
                view_layer: ViewLayers::Planetoids
            },
            displayable_collision_marker: CollisionMarker::<Displayable>::new(Shape::Circle(
                CircleData {
                    location: position,
                    radius: radius,
                },
            )),
            position: PositionComponent { position: position },
        })
    }
}

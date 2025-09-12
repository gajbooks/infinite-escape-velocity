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

const ANGULAR_VELOCITY_PLACEHOLDER: f32 = std::f32::consts::PI / 2.0;
const MAXIMUM_SPEED_PLACEHOLDER: f32 = 200.0;
const MAXIMUM_ACCELERATION_PLACEHOLDER: f32 = 40.0;
const MAXIMUM_HULL_PLACEHOLDER: Health = 100.0;
const HULL_REGEN_PLACEHOLDER: Health = 1.0;
const MAXIMUM_SHIELD_PLACEHOLDER: Health = 0.0;
const SHIELD_REGEN_PLACEHOLDER: Health = 0.0;

use crate::{
    backend::{shape::{PointData, Shape}, world_objects::components::{angular_velocity_properties_component::AngularVelocityPropertiesComponent, health_properties_component::HealthPropertiesComponent, health_state_component::HealthStateComponent, maximum_acceleration_properties_component::MaximumAccelerationPropertiesComponent, maximum_speed_properties_component::MaximumSpeedPropertiesComponent, semi_newtonian_physics_state_component::SemiNewtonianPhysicsStateComponent}},
    connectivity::{asset_index::AssetIndex, view_layers::ViewLayers},
    shared_types::{AccelerationScalar, AngularVelocity, Coordinates, Health, Rotation, Speed, Velocity},
};

use super::{
    components::{
        angular_velocity_state_component::AngularVelocityStateComponent, collision_component::CollisionSourceComponent,
        position_component::PositionComponent, rotation_component::RotationComponent,
        velocity_component::VelocityComponent,
    },
    server_viewport::Displayable,
};

#[derive(Bundle)]
pub struct ShipStateBundle {
    pub angular_velocity: AngularVelocityStateComponent,
    pub displayable_collision_marker: CollisionSourceComponent<Displayable>,
    pub health: HealthStateComponent,
    pub position: PositionComponent,
    pub rotation: RotationComponent,
    pub semi_newtonian_physics: SemiNewtonianPhysicsStateComponent,
    pub velocity: VelocityComponent,
}

#[derive(Bundle)]
pub struct ShipDataBundle {
    pub angular_velocity_properties: AngularVelocityPropertiesComponent,
    pub displayable: Displayable,
    pub health_properties: HealthPropertiesComponent,
    pub maximum_acceleration_properties: MaximumAccelerationPropertiesComponent,
    pub maximum_speed_properties: MaximumSpeedPropertiesComponent
}

#[derive(Bundle)]
pub struct ShipBundle {
    data: ShipDataBundle,
    state: ShipStateBundle,
}

impl ShipBundle {
    pub fn new_state_bundle(
        acceleration: Option<AccelerationScalar>,
        angular_velocity: Option<AngularVelocity>,
        hull: Health,
        position: Coordinates,
        rotation: Option<Rotation>,
        shield: Health,
        velocity: Option<Velocity>,
    ) -> ShipStateBundle {
        ShipStateBundle {
            angular_velocity: AngularVelocityStateComponent {
                angular_velocity: angular_velocity.unwrap_or_default(),
            },
            displayable_collision_marker: CollisionSourceComponent::<Displayable>::new(Shape::Point(
                PointData { point: position },
            )),
            health: HealthStateComponent { hull: hull, shield: shield },
            position: PositionComponent { position: position },
            rotation: RotationComponent {
                rotation: rotation.unwrap_or_default(),
            },
            semi_newtonian_physics: SemiNewtonianPhysicsStateComponent { thrust: acceleration.unwrap_or_default() },
            velocity: VelocityComponent {
                velocity: velocity.unwrap_or_default(),
            },
        }
    }

    pub fn new_data_bundle(asset_index: &AssetIndex) -> Result<ShipDataBundle, ()> {
        // Ship records do not exist yet in this build so just use a fixed asset name for now
        let asset_name = "default_asset";
        let display_asset = match asset_index.lookup_asset_by_name(asset_name) {
            Some(has) => *has,
            None => {
                tracing::warn!(
                    "Attempted to create ship with asset index {} which has an id which does not exist",
                    asset_name
                );
                return Err(());
            }
        };

        Ok(ShipDataBundle {
            displayable: Displayable {
                display_radius: 25.0,
                object_asset: display_asset,
                view_layer: ViewLayers::Ships,
            },
            angular_velocity_properties: AngularVelocityPropertiesComponent { maximum_angular_velocity: euclid::Angle { radians: ANGULAR_VELOCITY_PLACEHOLDER } },
            health_properties: HealthPropertiesComponent { maximum_hull: MAXIMUM_HULL_PLACEHOLDER, hull_regeneration_rate: HULL_REGEN_PLACEHOLDER, maximum_shield: MAXIMUM_SHIELD_PLACEHOLDER, shield_regeneration_rate: SHIELD_REGEN_PLACEHOLDER },
            maximum_acceleration_properties: MaximumAccelerationPropertiesComponent { maximum_acceleration: AccelerationScalar::new(MAXIMUM_ACCELERATION_PLACEHOLDER) },
            maximum_speed_properties: MaximumSpeedPropertiesComponent { maximum_speed: Speed::new(MAXIMUM_SPEED_PLACEHOLDER) }
        })
    }

    pub fn new(
        asset_index: &AssetIndex,
        position: Coordinates,
        angular_velocity: Option<AngularVelocity>,
        rotation: Option<Rotation>,
        velocity: Option<Velocity>,
    ) -> Result<Self, ()> {
        let data = Self::new_data_bundle(asset_index);

        let data = match data {
            Ok(valid) => valid,
            Err(()) => {
                return Err(());
            }
        };

        let state = Self::new_state_bundle(None, angular_velocity, MAXIMUM_HULL_PLACEHOLDER, position, rotation, MAXIMUM_SHIELD_PLACEHOLDER, velocity);

        Ok(Self { data, state })
    }
}

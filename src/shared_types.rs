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

use std::sync::atomic::*;
use euclid::*;

pub type IdType = u32;
pub type AtomicIdType = AtomicU32;
pub type GlobalCoordinateType = f64;
pub type LocalCoordinateType = f32;
pub type DeltaTimeType = f32;

pub struct WorldCoordinates;
pub struct RotationCoordinates;
pub struct VelocityCoordinates;
pub struct AccelerationCoordinates;

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum ObjectType {
    Unknown(),
    Viewport(),
    AIViewport(),
    AreaViewport(),
    WorldObject(IdType)
}

pub type AABB = Box2D<GlobalCoordinateType, WorldCoordinates>;

pub type Coordinates = Point2D<GlobalCoordinateType, WorldCoordinates>;
pub type Velocity = Vector2D<LocalCoordinateType, VelocityCoordinates>;
pub type Acceleration = Vector2D<LocalCoordinateType, AccelerationCoordinates>;
pub type Rotation = Angle<LocalCoordinateType>;
pub type AngularVelocity = Angle<LocalCoordinateType>;
pub type Radius = Length<GlobalCoordinateType, WorldCoordinates>;
pub type Distance = Length<GlobalCoordinateType, WorldCoordinates>;
pub type Offset = Size2D<GlobalCoordinateType, WorldCoordinates>;

pub type DeltaTA = Scale<DeltaTimeType, AccelerationCoordinates, VelocityCoordinates>;
pub type DeltaT = Scale<DeltaTimeType, VelocityCoordinates, WorldCoordinates>;

pub fn delta_t_to_delta_t_a(delta_t: DeltaT) -> DeltaTA {
    return DeltaTA::new(delta_t.get());
}

#[derive(Clone)]
pub struct CoordinatesRotation {
    pub location: Coordinates,
    pub rotation: Rotation
}

#[derive(Clone)]
pub struct CoordinatesVelocity {
    pub location: Coordinates,
    pub rotation: Rotation,
    pub velocity: Velocity,
    pub angular_velocity: AngularVelocity
}
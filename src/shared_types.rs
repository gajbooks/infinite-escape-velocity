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

use euclid::*;
use serde::Serialize;
use ts_rs::TS;

pub type GlobalCoordinateType = f64;
pub type LocalCoordinateType = f32;

pub struct WorldCoordinates;
pub struct RotationCoordinates;
pub struct VelocityCoordinates;
pub struct AccelerationCoordinates;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, TS)]
#[ts(export, export_to = "webapp/bindings/")]
pub struct ObjectType {
    name: String,
}

pub type AABB = Box2D<GlobalCoordinateType, WorldCoordinates>;

pub type Coordinates = Point2D<GlobalCoordinateType, WorldCoordinates>;
pub type Velocity = Vector2D<LocalCoordinateType, VelocityCoordinates>;
pub type Acceleration = Vector2D<LocalCoordinateType, AccelerationCoordinates>;
pub type Rotation = Angle<LocalCoordinateType>;
pub type AngularVelocity = Angle<LocalCoordinateType>;
pub type Radius = Length<GlobalCoordinateType, WorldCoordinates>;
pub type Distance = Length<GlobalCoordinateType, WorldCoordinates>;
pub type Speed = Length<LocalCoordinateType, VelocityCoordinates>;
pub type Offset = Size2D<GlobalCoordinateType, WorldCoordinates>;
pub type AccelerationScalar = Length<LocalCoordinateType, AccelerationCoordinates>;

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

use std::ops::Sub;

use crate::backend::spatial_optimizer::aabb_iterator::*;
use crate::shared_types::*;
use euclid::*;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct CircleData {
    pub location: Coordinates,
    pub radius: Radius,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct RoundedTubeData {
    pub center: Coordinates,
    pub rotation: Rotation,
    pub length: Distance,
    pub radius: Radius,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct PointData {
    pub point: Coordinates,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Shape {
    Circle(CircleData),
    RoundedTube(RoundedTubeData),
    Point(PointData),
}

impl Shape {
    pub fn move_center(&self, center: Coordinates) -> Shape {
        match self {
            Shape::Circle(circle) => Shape::Circle(CircleData {
                location: center,
                radius: circle.radius,
            }),
            Shape::RoundedTube(tube) => {
                Shape::RoundedTube(RoundedTubeData {
                    center: center,
                    rotation: tube.rotation,
                    length: tube.length,
                    radius: tube.radius,
                })
            }
            Shape::Point(_point) => Shape::Point(PointData { point: center }),
        }
    }

    pub fn set_rotation(&self, rotation: Rotation) -> Shape {
        match self {
            Shape::Circle(circle) => Shape::Circle(circle.clone()),
            Shape::RoundedTube(tube) => {
                Shape::RoundedTube(RoundedTubeData {
                    rotation: rotation.signed(),
                    center: tube.center,
                    length: tube.length,
                    radius: tube.radius,
                })
            }
            Shape::Point(point) => Shape::Point(point.clone()),
        }
    }

    pub fn aabb(&self) -> AABB {
        match self {
            Shape::Circle(circle) => AABB::new(
                Coordinates::new(
                    circle.location.x - circle.radius.get(),
                    circle.location.y - circle.radius.get(),
                ),
                Coordinates::new(
                    circle.location.x + circle.radius.get(),
                    circle.location.y + circle.radius.get(),
                ),
            ),
            Shape::RoundedTube(tube) => {
                let (point_1, point_2) = rounded_tube_points(&tube.center, &tube.rotation, &tube.length);
                let (min, max) = max_and_min_points(&point_1, &point_2);
                let radius = Vector2D::splat(tube.radius.get());

                AABB::new(min - radius, max + radius)
            }
            Shape::Point(point) => AABB::new(point.point, point.point),
        }
    }

    pub fn center(&self) -> Coordinates {
        match self {
            Shape::Circle(circle) => circle.location,
            Shape::RoundedTube(tube) => tube.center,
            Shape::Point(point) => point.point,
        }
    }

    pub fn aabb_iter(&self, hash_cell_size: u32) -> AABBIterator {
        return AABBIterator::new(self.aabb(), hash_cell_size);
    }

    pub fn collides(&self, other: &Shape) -> bool {
        match self {
            Shape::Circle(circle1) => match other {
                Shape::Circle(circle2) => circle_circle(circle1, circle2),
                Shape::RoundedTube(tube2) => circle_rounded_tube(circle1, tube2),
                Shape::Point(point2) => point_circle(point2, circle1),
            },
            Shape::RoundedTube(tube1) => match other {
                Shape::Circle(circle2) => circle_rounded_tube(circle2, tube1),
                Shape::RoundedTube(tube2) => tube_tube(tube1, tube2),
                Shape::Point(point2) => point_rounded_tube(point2, tube1),
            },
            Shape::Point(point1) => match other {
                Shape::Circle(circle2) => point_circle(point1, circle2),
                Shape::RoundedTube(tube2) => point_rounded_tube(point1, tube2),
                Shape::Point(point2) => point_point(point1, point2),
            },
        }
    }
}

fn max_and_min_points(point_1: &Coordinates, point_2: &Coordinates) -> (Coordinates, Coordinates) {
    (Coordinates::new(point_1.x.min(point_2.x), point_1.y.min(point_2.y)),
    Coordinates::new(point_1.x.max(point_2.x), point_1.y.max(point_2.y)))
}

fn rounded_tube_points(center: &Coordinates, rotation: &Rotation, length: &Distance) -> (Coordinates, Coordinates) {
    let point_1 = *center + Vector2D::<GlobalCoordinateType, WorldCoordinates>::from_angle_and_length(rotation.cast::<GlobalCoordinateType>(), length.get() / 2.0);
    let point_2 = *center + Vector2D::<GlobalCoordinateType, WorldCoordinates>::from_angle_and_length(rotation.sub(Rotation::pi()).cast::<GlobalCoordinateType>(), length.get() / 2.0);
    (point_1, point_2)
}

fn dist(point_1: Coordinates, point_2: Coordinates) -> Distance {
    Distance::new(point_1.distance_to(point_2))
}

fn circle_circle(circle1: &CircleData, circle2: &CircleData) -> bool {
    return circle1.location.distance_to(circle2.location)
        <= (circle1.radius + circle2.radius).get();
}

fn circle_rounded_tube(circle: &CircleData, tube: &RoundedTubeData) -> bool {
    let (point_1, point_2) = rounded_tube_points(&tube.center, &tube.rotation, &tube.length);
    let line_length = dist(point_1, point_2);
    let line_length_squared = line_length * line_length.get();

    if line_length_squared.get() <= 0.0 {
        return dist(circle.location, point_1)
            <= (circle.radius + tube.radius);
    }

    let t = ((circle.location.x - point_1.x) * (point_2.x - point_1.x)
        + (circle.location.y - point_1.y) * (point_2.y - point_1.y))
        / line_length_squared.get();
    let t = t.clamp(0.0, 1.0);

    let k = Coordinates::new(
        point_1.x + t * (point_2.x - point_1.x),
        point_1.y + t * (point_2.y - point_1.y),
    );
    return dist(circle.location, k) <= (circle.radius + tube.radius);
}

fn tube_tube(tube_1: &RoundedTubeData, tube_2: &RoundedTubeData) -> bool {
    let (tube_1_point_1, tube_1_point_2) = rounded_tube_points(&tube_1.center, &tube_1.rotation, &tube_1.length);
    let (tube_2_point_1, tube_2_point_2) = rounded_tube_points(&tube_2.center, &tube_2.rotation, &tube_2.length);
    let t1p1_to_t2 = circle_rounded_tube(
        &CircleData {
            location: tube_1_point_1,
            radius: tube_1.radius,
        },
        tube_2,
    );
    let t1p2_to_t2 = circle_rounded_tube(
        &CircleData {
            location: tube_1_point_2,
            radius: tube_1.radius,
        },
        tube_2,
    );
    let t2p1_to_t1 = circle_rounded_tube(
        &CircleData {
            location: tube_2_point_1,
            radius: tube_2.radius,
        },
        tube_1,
    );
    let t2p2_to_t1 = circle_rounded_tube(
        &CircleData {
            location: tube_2_point_2,
            radius: tube_2.radius,
        },
        tube_1,
    );

    return t1p1_to_t2 || t1p2_to_t2 || t2p1_to_t1 || t2p2_to_t1;
}

fn point_circle(point: &PointData, circle: &CircleData) -> bool {
    return circle.location.distance_to(point.point) < circle.radius.get();
}

fn point_rounded_tube(point: &PointData, tube: &RoundedTubeData) -> bool {
    return circle_rounded_tube(
        &CircleData {
            location: point.point,
            radius: Length::new(0.0),
        },
        tube,
    );
}

fn point_point(point1: &PointData, point2: &PointData) -> bool {
    point1.point == point2.point
}

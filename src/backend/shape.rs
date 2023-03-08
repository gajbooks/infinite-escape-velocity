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

use crate::backend::spatial_optimizer::aabb_iterator::*;
use crate::shared_types::*;
use euclid::*;

#[derive(Clone, Debug)]
pub struct CircleData {
    pub location: Coordinates,
    pub radius: Radius
}

#[derive(Clone, Debug)]
pub struct RoundedTubeData {
    pub point_1: Coordinates,
    pub point_2: Coordinates,
    pub radius: Radius
}

#[derive(Clone, Debug)]
pub enum Shape {
    Circle (CircleData),
    RoundedTube (RoundedTubeData)
}

impl Shape {
    pub fn move_center(&self, center: Coordinates) -> Shape {
        match self {
            Shape::Circle(circle) => Shape::Circle(CircleData{location: center, radius: circle.radius}),
            Shape::RoundedTube(tube) => {
                let old_center = self.center();
                let center_offset = center - old_center;
                Shape::RoundedTube(RoundedTubeData{point_1: tube.point_1 + center_offset, point_2: tube.point_2 + center_offset, radius: tube.radius}) 
            }
        }
    }

    pub fn aabb(&self) -> AABB {
        match self {
            Shape::Circle(circle) => AABB::new(Coordinates::new(circle.location.x - circle.radius.get(), circle.location.y - circle.radius.get()), Coordinates::new(circle.location.x + circle.radius.get(), circle.location.y + circle.radius.get())),
            Shape::RoundedTube(tube) => {
                let min = tube.point_1.min(tube.point_2);
                let max = tube.point_1.max(tube.point_2);
                let radius = Vector2D::new(tube.radius.get(), tube.radius.get());

                AABB::new(min - radius, max + radius)
            }
        }
    }

    pub fn center(&self) -> Coordinates {
        match self {
            Shape::Circle(circle) => circle.location,
            Shape::RoundedTube(tube) => {
                tube.point_1.lerp(tube.point_2, 0.5)
            }
        }
    }

    pub fn aabb_iter(&self) -> AABBIterator {
        return AABBIterator::new(self.aabb())
    }

    pub fn collides(&self, other: &Shape) -> bool {
        match self {
            Shape::Circle(circle1) => {
                match other {
                    Shape::Circle(circle2) => {
                        circle_circle(circle1, circle2)
                    },
                    Shape::RoundedTube(tube2) => {
                        circle_rounded_tube(circle1, tube2)
                    }
                }
            },
            Shape::RoundedTube(tube1) => {
                match other {
                    Shape::Circle(circle2) => {
                        circle_rounded_tube(circle2, tube1)
                    },
                    Shape::RoundedTube(tube2) => {
                        tube_tube(tube1, tube2)
                    }
                }
            }
        }
    }
}

fn dist_squared(point1: (f64, f64), point2: (f64, f64)) -> f64 {
    let dist_x = point1.0 - point2.0;
    let dist_y = point1.1 - point2.1;
    return dist_x.powi(2) + dist_y.powi(2);
}

fn dist(point1: (f64, f64), point2: (f64, f64)) -> f64 {
    return dist_squared(point1, point2).sqrt();
}

fn circle_circle(circle1: &CircleData, circle2: &CircleData) -> bool {
    return circle1.location.distance_to(circle2.location) <= (circle1.radius + circle2.radius).get();
}

fn circle_rounded_tube(circle: &CircleData, tube: &RoundedTubeData) -> bool {
    let line_length_squared = dist_squared(tube.point_1.to_tuple(), tube.point_2.to_tuple());

    if line_length_squared <= 0.0 {
        return dist(circle.location.to_tuple(), tube.point_1.to_tuple()) <= (circle.radius + tube.radius).get();
    }

    let t = ((circle.location.x - tube.point_1.x) * (tube.point_2.x - tube.point_1.x) + (circle.location.y - tube.point_1.y) * (tube.point_2.y - tube.point_1.y)) / line_length_squared;
    let t = t.clamp(0.0, 1.0);

    let k = (tube.point_1.x + t * (tube.point_2.x - tube.point_1.x), tube.point_1.y + t * (tube.point_2.y - tube.point_1.y));
    return dist((circle.location.x, circle.location.y), k) <= (circle.radius + tube.radius).get();
}

fn tube_tube(tube1: &RoundedTubeData, tube2: &RoundedTubeData) -> bool {
    let t1p1_to_t2 = circle_rounded_tube(&CircleData{location: tube1.point_1, radius: tube1.radius}, tube2);
    let t1p2_to_t2 = circle_rounded_tube(&CircleData{location: tube1.point_2, radius: tube1.radius}, tube2);
    let t2p1_to_t1 = circle_rounded_tube(&CircleData{location: tube2.point_1, radius: tube2.radius}, tube1);
    let t2p2_to_t1 = circle_rounded_tube(&CircleData{location: tube2.point_2, radius: tube2.radius}, tube1);

    return t1p1_to_t2 || t1p2_to_t2 || t2p1_to_t1 || t2p2_to_t1;
}
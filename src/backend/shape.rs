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

use super::aabb_iterator::*;
use super::super::shared_types::*;

#[derive(Clone, Debug)]
pub struct CircleData {
    pub location: Coordinates,
    pub radius: Radius
}

#[derive(Clone, Debug)]
pub struct RoundedTubeData {
    pub location: AABB,
    pub radius: Radius
}

#[derive(Clone, Debug)]
pub enum Shape {
    Circle (CircleData),
    RoundedTube (RoundedTubeData)
}

impl Shape {
    pub fn aabb(&self) -> AABB {
        match self {
            Shape::Circle(circle) => AABB::new(Coordinates::new(circle.location.x - circle.radius.get(), circle.location.y - circle.radius.get()), Coordinates::new(circle.location.x + circle.radius.get(), circle.location.y + circle.radius.get())),
            Shape::RoundedTube(tube) => {
                AABB::new(Coordinates::new(tube.location.min.x - tube.radius.get(), tube.location.min.y - tube.radius.get()), Coordinates::new(tube.location.max.x + tube.radius.get(), tube.location.max.y + tube.radius.get()))
            }
        }
    }

    pub fn center(&self) -> Coordinates {
        match self {
            Shape::Circle(circle) => circle.location,
            Shape::RoundedTube(tube) => {
                tube.location.center()
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
    let line_length_squared = dist_squared((tube.location.min.x, tube.location.min.y), (tube.location.max.x, tube.location.max.y));

    if line_length_squared <= 0.0 {
        return dist((circle.location.x, circle.location.y), (tube.location.min.x, tube.location.min.y)) <= (circle.radius + tube.radius).get();
    }

    let t = ((circle.location.x - tube.location.min.x) * (tube.location.max.x - tube.location.min.x) + (circle.location.y - tube.location.min.y) * (tube.location.max.y - tube.location.min.y)) / line_length_squared;
    let t = t.min(1.0).max(0.0);

    let k = (tube.location.min.x + t * (tube.location.max.x - tube.location.min.x), tube.location.min.y + t * (tube.location.max.y - tube.location.min.y));
    return dist((circle.location.x, circle.location.y), k) <= (circle.radius + tube.radius).get();
}

fn tube_tube(tube1: &RoundedTubeData, tube2: &RoundedTubeData) -> bool {
    let t1p1_to_t2 = circle_rounded_tube(&CircleData{location: tube1.location.min, radius: tube1.radius}, tube2);
    let t1p2_to_t2 = circle_rounded_tube(&CircleData{location: tube1.location.max, radius: tube1.radius}, tube2);
    let t2p1_to_t1 = circle_rounded_tube(&CircleData{location: tube2.location.min, radius: tube2.radius}, tube1);
    let t2p2_to_t1 = circle_rounded_tube(&CircleData{location: tube2.location.max, radius: tube2.radius}, tube1);

    return t1p1_to_t2 || t1p2_to_t2 || t2p1_to_t1 || t2p2_to_t1;
}
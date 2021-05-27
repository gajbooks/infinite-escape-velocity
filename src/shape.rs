use super::aabb_iterator::*;

#[derive(Clone, Debug)]
pub struct CircleData {
    pub x: f64,
    pub y: f64,
    pub r: f64
}

#[derive(Clone, Debug)]
pub struct RoundedTubeData {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub r: f64
}

#[derive(Clone, Debug)]
pub enum Shape {
    Circle (CircleData),
    RoundedTube (RoundedTubeData)
}

impl Shape {
    pub fn aabb(&self) -> AABB {
        match self {
            Shape::Circle(circle) => (circle.x - circle.r, circle.y + circle.r, circle.x + circle.r, circle.y - circle.r),
            Shape::RoundedTube(tube) => {
                let min_x = tube.x1.min(tube.x2);
                let max_x = tube.x1.max(tube.x2);
                let min_y = tube.y1.min(tube.y2);
                let max_y = tube.y1.max(tube.y2);

                (min_x - tube.r, max_y + tube.r, max_x + tube.r, min_y - tube.r)
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
    return dist((circle1.x, circle1.y), (circle2.x, circle2.y)) <= (circle1.r + circle2.r);
}

fn circle_rounded_tube(circle: &CircleData, tube: &RoundedTubeData) -> bool {
    let line_length_squared = dist_squared((tube.x1, tube.y1), (tube.x2, tube.y2));

    if line_length_squared <= 0.0 {
        return dist((circle.x, circle.y), (tube.x1, tube.y1)) <= (circle.r + tube.r);
    }

    let t = ((circle.x - tube.x1) * (tube.x2 - tube.x1) + (circle.y - tube.y1) * (tube.y2 - tube.y1)) / line_length_squared;
    let t = t.min(1.0).max(0.0);

    let k = (tube.x1 + t * (tube.x2 - tube.x1), tube.y1 + t * (tube.y2 - tube.y1));
    return dist((circle.x, circle.y), k) <= (circle.r + tube.r);
}

fn tube_tube(tube1: &RoundedTubeData, tube2: &RoundedTubeData) -> bool {
    let t1p1_to_t2 = circle_rounded_tube(&CircleData{x: tube1.x1, y: tube1.y1, r: tube1.r}, tube2);
    let t1p2_to_t2 = circle_rounded_tube(&CircleData{x: tube1.x2, y: tube1.y2, r: tube1.r}, tube2);
    let t2p1_to_t1 = circle_rounded_tube(&CircleData{x: tube2.x1, y: tube2.y1, r: tube2.r}, tube1);
    let t2p2_to_t1 = circle_rounded_tube(&CircleData{x: tube2.x2, y: tube2.y2, r: tube2.r}, tube1);

    return t1p1_to_t2 || t1p2_to_t2 || t2p1_to_t1 || t2p2_to_t1;
}
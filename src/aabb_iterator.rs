use super::hash_coordinates::*;

pub type AABB = (f64, f64, f64, f64);
const SQUARE_SIZE: HashCoordinateType = 1;

pub struct AABBIterator {
    x_len: HashCoordinateType,
    end_x: HashCoordinateType,
    end_y: HashCoordinateType,
    current_x: HashCoordinateType,
    current_y: HashCoordinateType
}

impl AABBIterator {
    pub fn new(bb: AABB) -> AABBIterator {
        let max = HashCoordinateType::MAX as f64;
        let start_x = (bb.0 % max) as HashCoordinateType / SQUARE_SIZE;
        let start_y = (bb.3 % max) as HashCoordinateType / SQUARE_SIZE;
        let end_x = (bb.2 % max) as HashCoordinateType / SQUARE_SIZE;
        let end_y = (bb.1 % max) as HashCoordinateType / SQUARE_SIZE;

        let x_len = (end_x - start_x) + 1;
        return AABBIterator {x_len: x_len, end_x: end_x, end_y: end_y, current_x: start_x, current_y: start_y}
    }
}

impl Iterator for AABBIterator {
    type Item = HashCoordinates;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_y > self.end_y {
            return None;
        }
        else
        {
            let old_result = HashCoordinates {x: self.current_x, y: self.current_y};
            self.current_x += 1;
            let x_has_wrapped = ((self.x_len - 1) - (self.end_x - self.current_x)) / self.x_len;
            self.current_y += x_has_wrapped;
            self.current_x -= self.x_len * x_has_wrapped;
            return Some(old_result);
        }
    }
}
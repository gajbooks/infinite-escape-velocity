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
use super::hash_coordinates::*;
use crate::shared_types::*;

pub struct AABBIterator<const HASH_CELL_SIZE: u32> {
    x_len: HashCoordinateType,
    end_x: HashCoordinateType,
    end_y: HashCoordinateType,
    current_x: HashCoordinateType,
    current_y: HashCoordinateType,
}

impl<const HASH_CELL_SIZE: u32> AABBIterator<HASH_CELL_SIZE> {

    pub fn new(bb: AABB) -> AABBIterator<HASH_CELL_SIZE> {
        let max = HashCoordinateType::MAX as f64;
        let start_x = (bb.min.x % max) as HashCoordinateType / HASH_CELL_SIZE as i32;
        let start_y = (bb.min.y % max) as HashCoordinateType / HASH_CELL_SIZE as i32;
        let end_x = (bb.max.x % max) as HashCoordinateType / HASH_CELL_SIZE as i32;
        let end_y = (bb.max.y % max) as HashCoordinateType / HASH_CELL_SIZE as i32;

        let x_len = (end_x - start_x) + 1;
        return AABBIterator {
            x_len: x_len,
            end_x: end_x,
            end_y: end_y,
            current_x: start_x,
            current_y: start_y,
        };
    }
}

impl<const HASH_CELL_SIZE: u32> Iterator for AABBIterator<HASH_CELL_SIZE> {
    type Item = HashCoordinates;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_y > self.end_y {
            return None;
        } else {
            let old_result = HashCoordinates {
                x: self.current_x,
                y: self.current_y,
            };
            self.current_x += 1;
            let x_has_wrapped = ((self.x_len - 1) - (self.end_x - self.current_x)) / self.x_len;
            self.current_y += x_has_wrapped;
            self.current_x -= self.x_len * x_has_wrapped;
            return Some(old_result);
        }
    }
}

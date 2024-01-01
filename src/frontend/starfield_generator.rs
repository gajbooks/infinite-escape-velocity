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

use fxhash::*;
use macroquad::prelude::*;
use std::hash::{Hash, Hasher};

const STAR_TRANSPARENCY: u8 = 200;

const SPECTRA_TABLE: [Color; 25] = [
    color_u8!(255, 177, 14, STAR_TRANSPARENCY),
    color_u8!(255, 192, 44, STAR_TRANSPARENCY),
    color_u8!(255, 204, 76, STAR_TRANSPARENCY),
    color_u8!(255, 213, 105, STAR_TRANSPARENCY),
    color_u8!(255, 219, 131, STAR_TRANSPARENCY),
    color_u8!(255, 224, 153, STAR_TRANSPARENCY),
    color_u8!(255, 227, 171, STAR_TRANSPARENCY),
    color_u8!(255, 229, 187, STAR_TRANSPARENCY),
    color_u8!(255, 230, 200, STAR_TRANSPARENCY),
    color_u8!(255, 231, 211, STAR_TRANSPARENCY),
    color_u8!(255, 232, 221, STAR_TRANSPARENCY),
    color_u8!(255, 232, 229, STAR_TRANSPARENCY),
    color_u8!(255, 232, 236, STAR_TRANSPARENCY),
    color_u8!(255, 232, 243, STAR_TRANSPARENCY),
    color_u8!(255, 232, 248, STAR_TRANSPARENCY),
    color_u8!(255, 232, 253, STAR_TRANSPARENCY),
    color_u8!(252, 230, 255, STAR_TRANSPARENCY),
    color_u8!(248, 226, 255, STAR_TRANSPARENCY),
    color_u8!(245, 223, 255, STAR_TRANSPARENCY),
    color_u8!(242, 221, 255, STAR_TRANSPARENCY),
    color_u8!(240, 218, 255, STAR_TRANSPARENCY),
    color_u8!(238, 216, 255, STAR_TRANSPARENCY),
    color_u8!(236, 214, 255, STAR_TRANSPARENCY),
    color_u8!(234, 213, 255, STAR_TRANSPARENCY),
    color_u8!(232, 211, 255, STAR_TRANSPARENCY),
];

pub struct StarfieldGenerator {
    max_coordinate_divisor: f32,
    min_coordinate_divisor: f32,
    world_coordinates_per_cell: f32,
    min_size: f32,
    max_size: f32,
    star_list: Vec<StarData>,
}

struct StarData {
    screen_x: f32,
    screen_y: f32,
    speed_multiplier: f32,
    spectra: f32,
    size: f32,
}

impl StarfieldGenerator {
    pub fn new(
        max_coordinate_divisor: f32,
        min_coordinate_divisor: f32,
        world_coordinates_per_cell: f32,
        min_size: f32,
        max_size: f32,
    ) -> StarfieldGenerator {
        StarfieldGenerator {
            max_coordinate_divisor: max_coordinate_divisor,
            min_coordinate_divisor: min_coordinate_divisor,
            world_coordinates_per_cell: world_coordinates_per_cell,
            min_size: min_size,
            max_size: max_size,
            star_list: Vec::new(),
        }
    }

    fn simple_xorshift(state: u64) -> u64 {
        let mut x = state;
        x ^= x.wrapping_shl(13);
        x ^= x.wrapping_shr(7);
        x ^= x.wrapping_shl(17);
        return x;
    }

    fn zero_one_generator(random: u64) -> f32 {
        return random as f32 / u64::MAX as f32;
    }

    fn star_from_coordinates(&self, x: i64, y: i64) -> StarData {
        let mut s = FxHasher::default();
        x.hash(&mut s);
        y.hash(&mut s);
        let seed = s.finish();
        let x_offset = Self::zero_one_generator(seed);

        let seed = Self::simple_xorshift(seed);
        let y_offset = Self::zero_one_generator(seed);

        let seed = Self::simple_xorshift(seed);
        let speed_multiplier = Self::zero_one_generator(seed);

        let seed = Self::simple_xorshift(seed);
        let spectra = Self::zero_one_generator(seed);

        let seed = Self::simple_xorshift(seed);
        let size = Self::zero_one_generator(seed);

        return StarData {
            screen_x: (x as f32 - 1.0 + x_offset) * self.world_coordinates_per_cell,
            screen_y: (y as f32 - 1.0 + y_offset) * self.world_coordinates_per_cell,
            speed_multiplier: speed_multiplier,
            spectra: spectra,
            size: size,
        };
    }

    pub fn draw_stars(
        &mut self,
        x_coordinate: f32,
        y_coordinate: f32,
        screen_width: f32,
        screen_height: f32,
    ) {
        let x_coordinate = x_coordinate / self.max_coordinate_divisor;
        let y_coordinate = y_coordinate / self.max_coordinate_divisor;

        let start_x = (x_coordinate / self.world_coordinates_per_cell).floor();
        let start_y = (y_coordinate / self.world_coordinates_per_cell).floor();

        let cells_x = (screen_width / self.world_coordinates_per_cell) + 1.0;
        let cells_y = (screen_height / self.world_coordinates_per_cell) + 1.0;

        self.generate_stars(
            start_x as i64,
            start_y as i64,
            cells_x as i64,
            cells_y as i64,
        );

        let divisor_ratio = (self.max_coordinate_divisor / self.min_coordinate_divisor) - 1.0;

        let size_range = self.max_size - self.min_size;

        let max_x_to_draw = screen_width + self.max_size;
        let max_y_to_draw = screen_height + self.max_size;
        let min_x_to_draw = -self.max_size;
        let min_y_to_draw = -self.max_size;

        for star in &self.star_list {
            let offset_ratio = 1.0 + (divisor_ratio * star.speed_multiplier);
            let star_size = (star.size * size_range) + self.min_size;
            let star_x = (star.screen_x - x_coordinate) * offset_ratio;
            let star_y = (star.screen_y - y_coordinate) * offset_ratio;

            if star_x > min_x_to_draw
                && star_y > min_y_to_draw
                && star_x < max_x_to_draw
                && star_y < max_y_to_draw
            {
                let spectra_lookup = (star.spectra * SPECTRA_TABLE.len() as f32) as usize;
                draw_circle(star_x, star_y, star_size, SPECTRA_TABLE[spectra_lookup]);
            }
        }
    }

    fn generate_stars(&mut self, start_x: i64, start_y: i64, cells_x: i64, cells_y: i64) {
        self.star_list.clear();

        for i in 0..cells_y as i64 {
            for j in 0..cells_x as i64 {
                self.star_list
                    .push(self.star_from_coordinates(j + start_x as i64, i + start_y as i64));
            }
        }
    }
}

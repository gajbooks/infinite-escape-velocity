use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use macroquad::prelude::*;

pub struct StarfieldGenerator {
    coordinate_divisor: u16,
    seed_nonce: u32,
    density_offset: u64,
    world_coordinates_per_cell: f32
}

impl StarfieldGenerator {
    pub fn new(coordinate_divisor: u16, world_coordinates_per_cell: f32, seed_nonce: u32, density: f64) -> StarfieldGenerator {
        let density_offset = (u64::MAX as f64 * density).floor() as u64;

        StarfieldGenerator{coordinate_divisor: coordinate_divisor,
            seed_nonce: seed_nonce,
            density_offset: density_offset,
            world_coordinates_per_cell: world_coordinates_per_cell}
    }

    fn star_present(&self, x: i64, y: i64) -> bool {
        let mut s = DefaultHasher::new();
        x.hash(&mut s);
        y.hash(&mut s);
        self.seed_nonce.hash(&mut s);
        return s.finish() < self.density_offset;
    }

    pub fn generate_image(&self, x_coordinate: f64, y_coordinate: f64, screen_width: f32, screen_height: f32) {

        let start_x = (x_coordinate / self.world_coordinates_per_cell as f64).floor() as i64;
        let start_y = (y_coordinate / self.world_coordinates_per_cell as f64).floor() as i64;

        let cells_x = (screen_width / self.world_coordinates_per_cell) + 1.0;
        let cells_y = (screen_height / self.world_coordinates_per_cell) + 1.0;

        let starfield_x_origin = (start_x as f32 * self.world_coordinates_per_cell) as f64 - x_coordinate;
        let starfield_y_origin = (start_y as f32 * self.world_coordinates_per_cell) as f64 - y_coordinate;

        let starfield_x_pixels = cells_x * self.world_coordinates_per_cell;
        let starfield_y_pixels = cells_y * self.world_coordinates_per_cell;

        let mut grid_raw: Vec<bool> = vec![false; cells_x as usize * cells_y as usize];
        let mut grid_base: Vec<_> = grid_raw.as_mut_slice().chunks_exact_mut(cells_y as usize).collect();
        let grid = grid_base.as_mut_slice();

        let mut image = macroquad::texture::Image::gen_image_color(cells_x as u16, cells_y as u16, macroquad::color::BLACK);

        for i in 0..cells_y as i64 {
            for j in 0..cells_x as i64 {
                grid[j as usize][i as usize] = self.star_present(j + start_x, i + start_y);
                if grid[j as usize][i as usize] {
                    image.set_pixel(j as u32, i as u32, macroquad::color::WHITE);
                }
            }
        }

        let texture = macroquad::texture::Texture2D::from_image(&image);
        texture.set_filter(FilterMode::Nearest);
        draw_texture_ex(texture, starfield_x_origin as f32, starfield_y_origin as f32, WHITE, DrawTextureParams {
            dest_size: Some(vec2(starfield_x_pixels, starfield_y_pixels)),
            source: None,
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            pivot: None
        });
        set_default_camera();
    }
}
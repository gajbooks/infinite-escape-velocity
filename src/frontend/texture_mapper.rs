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

use dashmap::DashMap;
use fxhash::FxBuildHasher;
use webp::*;
use std::sync::Arc;
use macroquad::texture::*;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct TextureReference {
    texture: Arc<Texture2D>
}

impl TextureReference {
    fn new(texture: Arc<Texture2D>) -> TextureReference {
        TextureReference{texture: texture}
    }

    pub fn get_texture(&self) -> &Texture2D {
        return &self.texture;
    }
}

pub struct TextureMapper {
    texture_map: DashMap<Arc<PathBuf>, Arc<Texture2D>, FxBuildHasher>,
    base_path: PathBuf
}

impl TextureMapper {
    pub fn new(base_path: &Path) -> TextureMapper {
        TextureMapper{
            texture_map: DashMap::with_hasher(FxBuildHasher::default()),
            base_path: PathBuf::from(base_path)}
    }

    pub fn get_texture_reference(&self, path: &Path) -> Option<TextureReference> {
        let canonicalized = match fs::canonicalize(self.base_path.join(path)) {
            Ok(valid_path) => Arc::new(valid_path),
            Err(path_error) => {
                println!("Error getting full path for file {:?} with error {}", path, path_error);
                return None;
            }
        };
        
        let texture = match self.texture_map.get(&canonicalized) {
            Some(texture) => texture.clone(),
            None => {return None}
        };

        return Some(TextureReference::new(texture));
    }

    pub fn load_texture(&self, filename: &Path) -> Result<TextureReference, ()>{
        let full_path = match fs::canonicalize(self.base_path.join(filename)) {
            Ok(valid_path) => Arc::new(valid_path),
            Err(path_error) => {
                println!("Error getting full path for file {:?} with error {}", filename, path_error);
                return Err(());
            }
        };

        let file_data = match fs::read(&*full_path) {
            Ok(data) => data,
            Err(e) => {println!("Error loading texture {:?} with error: {}", full_path, e); return Err(())}
        };

        let webp_decoder = Decoder::new(&file_data);

        let image = match webp_decoder.decode() {
            Some(decoded) => decoded,
            None => {println!("Image could not be decoded as WebP: {:?}", filename); return Err(())}
        };

        let width = image.width() as u16;
        let height = image.width() as u16;

        let image_data: Vec<u8>;

        // Do this idiotic test and conversion because Webp crate doesn't expose image type and crashes Macroquad
        if image.len() == (image.width() * image.height() * 3) as usize {
            // Convert RGB to RGBA
            image_data = image.chunks_exact(3).map(|x| [x[0], x[1], x[2], u8::MAX]).flatten().collect::<Vec<_>>();
        } else if image.len() == (image.width() * image.height() * 4) as usize {
            // Convert RGBA out of reference
            image_data = image.chunks_exact(4).map(|x| [x[0], x[1], x[2], x[3]]).flatten().collect::<Vec<_>>();
        } else {
            // Those are the only two options, so everything else is invalid
            println!("WebP image is not RGB or RGBA: {:?}", filename);
            return Err(());
        }

        let new_texture = Texture2D::from_rgba8(width, height, &image_data);
        self.texture_map.insert(full_path.clone(), Arc::new(new_texture));

        match self.get_texture_reference(filename) {
            Some(texture) => {
                return Ok(texture);
            },
            None => {
                println!("Could not find immediately loaded texture reference: {:?}", filename);
                return Err(());
            }
        };
    }

    pub fn atlasize_textures() {
        build_textures_atlas();
    }
}
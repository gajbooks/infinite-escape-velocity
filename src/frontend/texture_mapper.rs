use dashmap::DashMap;
use fxhash::FxBuildHasher;
use webp::*;
use std::sync::Arc;
use macroquad::texture::*;
use std::fs;

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
    texture_map: DashMap<String, Arc<Texture2D>, FxBuildHasher>
}

impl TextureMapper {
    pub fn new() -> TextureMapper {
        TextureMapper{texture_map: DashMap::with_hasher(FxBuildHasher::default())}
    }

    fn insert_texture(&self, name: &str, texture: Texture2D) {
        self.texture_map.insert(name.to_owned(), Arc::new(texture));
    }

    pub fn get_texture_reference(&self, name: &str) -> Option<TextureReference> {
        let texture = match self.texture_map.get(name) {
            Some(texture) => texture.clone(),
            None => {return None}
        };

        return Some(TextureReference::new(texture));
    }

    pub fn load_texture(&self, name: &str, filename: &str) -> Result<(), ()>{
        let file_data = match fs::read(filename) {
            Ok(data) => data,
            Err(e) => {println!("Error loading texture {} with error: {}", filename, e); return Err(())}
        };

        let webp_decoder = Decoder::new(&file_data);

        let image = match webp_decoder.decode() {
            Some(decoded) => decoded,
            None => {println!("Image could not be decoded as WebP: {}", filename); return Err(())}
        };

        let width = image.width() as u16;
        let height = image.width() as u16;

        let new_texture = Texture2D::from_rgba8(width, height, &image);
        self.insert_texture(name, new_texture);
        return Ok(());
    }

    pub fn atlasize_textures() {
        build_textures_atlas();
    }
}
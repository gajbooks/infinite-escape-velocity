use super::super::shared_types::*;
use macroquad::prelude::*;
use super::texture_mapper::*;

#[derive(Clone)]
pub struct MappedTexture {
    object_type: ObjectType,
    texture: TextureReference
}

impl MappedTexture {
    pub fn verify_type(&self, verify_type: &ObjectType) -> bool {
        return verify_type.eq(&self.object_type);
    }

    pub fn get_texture(&self) -> &Texture2D {
        self.texture.get_texture()
    }
}

pub struct ObjectIndex {
    texture_mapper: TextureMapper
}

impl ObjectIndex {
    pub fn new(texture_mapper: TextureMapper) -> ObjectIndex {
        ObjectIndex{texture_mapper: texture_mapper}
    }

    pub fn map_object_type_to_texture(&self, object_type: &ObjectType) -> MappedTexture {
        let new_texture = self.texture_mapper.get_texture_reference("default").unwrap();
        return MappedTexture{object_type: object_type.to_owned(), texture: new_texture};
    }
}
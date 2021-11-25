use super::super::shared_types::*;
use super::texture_mapper::*;
use std::sync::*;
use super::super::configuration_loaders::{dynamic_object_configuration::*, object_type_map::*};
use std::path::{Path, PathBuf};
use macroquad::texture::*;

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

pub struct ObjectToTextureIndex {
    texture_mapper: Arc<TextureMapper>,
    type_mapper: Arc<ObjectTypeMap>,
    loaded_configuration: Arc<DynamicObjectConfiguration>,
    default_texture: Arc<PathBuf>
}

impl ObjectToTextureIndex {
    pub fn new(
        texture_mapper: Arc<TextureMapper>,
        type_mapper: Arc<ObjectTypeMap>,
        loaded_configuration: Arc<DynamicObjectConfiguration>,
        default_texture: Arc<PathBuf>) -> ObjectToTextureIndex {
        ObjectToTextureIndex{texture_mapper: texture_mapper, type_mapper: type_mapper, loaded_configuration: loaded_configuration, default_texture: default_texture}
    }

    pub fn map_object_type_to_texture(&self, object_type: &ObjectType) -> Option<MappedTexture> {
        let types = match self.type_mapper.object_type_to_object_type_parameters(object_type) {
            Ok(has) => {
                match has {
                    Some(type_has_texture) => type_has_texture,
                    None => {
                        return None;
                    }
                }
            },
            Err(()) => {
                return None;
            }
        };

        let texture_filename = match self.loaded_configuration.get(&types) {
            Some(has) => {
                match has.graphics_parameters {
                    Some(graphics) => {
                        match graphics.simple {
                            Some(basic_texture) => {
                                Some(basic_texture.filename)
                            },
                            None => None
                        }
                    },
                    None => None
                }
            },
            None => None
        };

        match texture_filename {
            Some(exists) => {
                match self.texture_mapper.get_texture_reference(&Path::new(&exists)) {
                    Some(texture) => {
                        Some(MappedTexture{object_type: object_type.clone(), texture: texture})
                    },
                    None => {
                        Some(MappedTexture{object_type: object_type.clone(), texture: self.texture_mapper.get_texture_reference(&self.default_texture).unwrap()})
                    }
                }
            },
            None => Some(MappedTexture{object_type: object_type.clone(), texture: self.texture_mapper.get_texture_reference(&self.default_texture).unwrap()})
        }
    }
}
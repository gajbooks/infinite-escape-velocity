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

use super::super::configuration_loaders::{object_configuration::*, object_type_map::*};
use super::super::shared_types::*;
use super::texture_mapper::*;
use macroquad::texture::*;
use std::path::Path;
use std::sync::*;

#[derive(Clone)]
pub struct MappedTexture {
    object_type: ObjectType,
    texture: TextureReference,
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
    loaded_configuration: Arc<ObjectConfigurationMap>,
    default_texture: TextureReference,
}

impl ObjectToTextureIndex {
    pub fn new(
        texture_mapper: Arc<TextureMapper>,
        type_mapper: Arc<ObjectTypeMap>,
        loaded_configuration: Arc<ObjectConfigurationMap>,
        default_texture: TextureReference,
    ) -> ObjectToTextureIndex {
        ObjectToTextureIndex {
            texture_mapper: texture_mapper,
            type_mapper: type_mapper,
            loaded_configuration: loaded_configuration,
            default_texture: default_texture,
        }
    }

    pub fn map_object_type_to_texture(&self, object_type: &ObjectType) -> Option<MappedTexture> {
        let types = match self
            .type_mapper
            .object_type_to_object_type_parameters(object_type)
        {
            Ok(has) => match has {
                Some(type_has_texture) => type_has_texture,
                None => {
                    return None;
                }
            },
            Err(()) => {
                return None;
            }
        };

        let texture_filename = match self.loaded_configuration.get(&types) {
            Some(has) => {
                    match has.object.get_graphics_parameters() {
                        Some(graphics) => {
                            match &graphics.simple {
                                Some(basic_texture) => Some(basic_texture.filename.clone()),
                                None => None
                            }
                        }
                        None => None,
                    }
            }
            None => None,
        };

        match texture_filename {
            Some(exists) => {
                match self
                    .texture_mapper
                    .get_texture_reference(&Path::new(&exists))
                {
                    Some(texture) => Some(MappedTexture {
                        object_type: object_type.clone(),
                        texture: texture,
                    }),
                    None => Some(MappedTexture {
                        object_type: object_type.clone(),
                        texture: self.default_texture.clone(),
                    }),
                }
            }
            None => Some(MappedTexture {
                object_type: object_type.clone(),
                texture: self.default_texture.clone(),
            }),
        }
    }
}

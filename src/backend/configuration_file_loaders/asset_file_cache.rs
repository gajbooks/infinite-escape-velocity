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

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};
use bytes::Bytes;
use futures_util::future::join_all;

use crate::configuration_file_structures::{asset_definition_file::{
    AssetDefinition, AssetDefinitionFile, AssetType, GraphicsType, MetaAsset,
}, reference_string_types::AssetReference};

use super::{archive_readers::{archive_reader::ArchiveReader, filesystem_reader::FilesystemReader, zip_reader::ZipReader}, asset_bundle_loader::AssetBundle};

pub struct AssetFileCache {
    assets: HashMap<AssetReference, (AssetDefinition, Option<Bytes>)>,
}

impl AssetFileCache {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }

    pub fn get_asset_data_by_name(
        &self,
        asset_name: &str,
    ) -> Option<(AssetDefinition, Option<Bytes>)> {
        self.assets.get(asset_name).cloned()
    }

    pub fn verify_assets(&self) -> Result<(), ()> {
        self.assets.iter().filter_map(|(name, (asset_info, _data))| {
            // Potentially we will want to validate file extensions here as files which are compatible with web browsers or which respect their intended asset types, but for now it is unimportant
            match &asset_info.asset_type {
                AssetType::Meta(meta) => {
                    Some((name, meta))
                },
                _=> None
            }
        }).map(|(name, meta)| {
            match meta {
                MetaAsset::Graphics(graphics) => {
                    match graphics {
                        GraphicsType::SimpleSquareRotationalSpriteSheet { sprite_count_x: _, sprite_count_y: _, image_data_asset } => {
                            match self.assets.get(image_data_asset) {
                                Some((linked_info, _data)) => {
                                    match linked_info.asset_type {
                                        AssetType::Meta(_) => {
                                            // We will potentially invalidate this assumption in the future, but for now, meta-resources only need to reference data resources
                                            tracing::error!("Meta-asset {} cannot have another meta-asset {} as a data value", name, linked_info.asset_name);
                                            Err(())
                                        },
                                        _ => {
                                            Ok(())
                                        }
                                    }
                                },
                                None => {
                                    // We will potentially loosen this restriction in the future with regards to asset bundle loading, but for now it is enforced
                                    tracing::error!("Graphics meta-asset {} references an image asset {} which does not exist", name, image_data_asset);
                                    Err(())
                                },
                            }
                        }
                    }
                }
            }
        }).collect()
    }

    pub async fn load_asset_bundle(&mut self, file: &AssetBundle) -> Result<(), ()> {
        match &file.bundle_type {
            super::asset_bundle_loader::AssetBundleType::Folder => {
                match FilesystemReader::new(&file.path).await {
                    Ok(has) => self.load_asset_bundle_generic(file, has).await,
                    Err(()) => {
                        return Err(());
                    }
                }
            }
            super::asset_bundle_loader::AssetBundleType::Zipped(zip_type) => match zip_type {
                super::asset_bundle_loader::AssetBundleFileType::Zip => {
                    match ZipReader::new(&file.path).await {
                        Ok(has) => self.load_asset_bundle_generic(file, has).await,
                        Err(()) => {
                            return Err(());
                        }
                    }
                }
            },
        }
    }

    async fn load_asset_bundle_generic(
        &mut self,
        file: &AssetBundle,
        asset_loader: impl ArchiveReader,
    ) -> Result<(), ()> {
        let file_get_tasks = asset_loader
            .get_directories()
            .await
            .into_iter()
            .map(|possible_directory| (possible_directory, &asset_loader))
            .map(|(possible_directory, asset_loader)| async move {
                let possible_asset_file =
                    PathBuf::from(possible_directory.to_owned()).join("asset.json");
                match asset_loader.try_get_file(&possible_asset_file).await {
                    Ok(tried_to_read_asset_json) => match tried_to_read_asset_json {
                        Some(has_asset_json) => Ok(Some((possible_directory, has_asset_json))),
                        None => Ok(None),
                    },
                    Err(error_getting_asset_file) => Err(error_getting_asset_file),
                }
            });

        let mut error_reading_asset_json = false;

        let found_asset_files: Vec<_> = join_all(file_get_tasks)
            .await
            .into_iter()
            .filter_map(|found| match found {
                Ok(has) => has.to_owned(),
                Err(()) => {
                    error_reading_asset_json = true;
                    None
                }
            })
            .collect();

        let found_asset_files: Vec<_> = found_asset_files.into_iter().filter_map(|(containing_directory, asset_json_data)| {
            match serde_json::de::from_slice::<AssetDefinitionFile>(&asset_json_data) {
                Ok(deserialized) => Some((containing_directory, deserialized)),
                Err(error_deserializing) => {
                    tracing::error!("Error deserializing asset.json from asset bundle {} with path {} and error {}", file.path.to_string_lossy(), containing_directory.to_string_lossy(), error_deserializing);
                    error_reading_asset_json = true;
                    None
                },
            }
        }).collect();

        if error_reading_asset_json {
            tracing::error!(
                "Error reading all asset.json files from bundle {}",
                file.path.to_string_lossy()
            );
            return Err(());
        }

        let mut duplicate_name_checker = HashSet::<AssetReference>::new();

        for (_path, asset_info) in &found_asset_files {
            for asset in &asset_info.assets {
                if duplicate_name_checker.insert(asset.asset_name.clone()) == false {
                    tracing::warn!("Duplicated asset name found within bundle {} with name {}, this could be an error or result in inconsistent load orders!", file.path.to_string_lossy(), asset.asset_name);
                }
            }
        }

        let mut error_reading_asset_files = false;

        let read_asset_file_tasks = found_asset_files
            .into_iter()
            .flat_map(|(possible_directory, flatten_assets)| {
                flatten_assets
                    .assets
                    .into_iter()
                    .map(move |asset| (possible_directory.clone(), asset))
            })
            .map(|possible_directory| (possible_directory, &asset_loader))
            .map(
                |((containing_directory, asset_definition), asset_loader)| async move {
                    let name = asset_definition.asset_type.try_get_filename();

                    match name {
                        Some(load_file) => {
                            let loaded = asset_loader
                                .try_get_file(
                                    &PathBuf::from(&containing_directory).join(&load_file),
                                )
                                .await;

                            (asset_definition, loaded)
                        }
                        None => (asset_definition, Ok(None)),
                    }
                },
            );

        let read_asset_files: Vec<_> = join_all(read_asset_file_tasks).await.into_iter().map(|(asset_definition, asset_data)| {
            match asset_data {
                Ok(read) => {
                    match read {
                        Some(has_file) => {
                            Some((asset_definition, Some(has_file)))
                        },
                        None => {
                            match asset_definition.asset_type {
                                AssetType::Meta(ref _meta) => {
                                    // Asset type is meta, no associated data should be present
                                    Some((asset_definition, None))
                                },
                                _ => {
                                    // Non-meta asset, means that a file could not be read
                                    tracing::error!("Associated asset file {} does not exist for asset {}", asset_definition.asset_type.try_get_filename().unwrap_or_default(), &asset_definition.asset_name);
                                    error_reading_asset_files = true;
                                    None
                                }
                            }
                        },
                    }
                },
                Err(()) => {
                    tracing::error!("File read error reading asset data file {} from bundle for asset specified as {}", asset_definition.asset_type.try_get_filename().unwrap_or_default(), &asset_definition.asset_name);
                    error_reading_asset_files = true;
                    None
                },
            }
        }).filter_map(|x| x).collect();

        if error_reading_asset_files {
            tracing::error!(
                "Error reading associated asset files for asset.json files from bundle {}",
                file.path.to_string_lossy()
            );
            return Err(());
        }

        for (asset_info, data) in read_asset_files {
            tracing::trace!("Loaded asset {}", asset_info.asset_name);
            self.assets
                .insert(asset_info.asset_name.clone(), (asset_info, data));
        }

        Ok(())
    }
}

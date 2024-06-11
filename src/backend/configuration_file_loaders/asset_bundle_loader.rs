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

use std::{collections::HashSet, path::PathBuf};

use futures_util::future::join_all;

use crate::configuration_file_structures::load_order_file::LoadOrderFile;

pub enum AssetBundleFileType {
    Zip
}

impl AssetBundleFileType {
    fn from_file_extension(ext: &str) -> Option<Self> {
        match ext {
            "zip" => Some(AssetBundleFileType::Zip),
            _ => None
        }
    }
}

pub struct AssetBundle {
    pub bundle_type: AssetBundleType,
    pub name: String,
    pub path: PathBuf
}

pub enum AssetBundleType {
    Folder,
    Zipped(AssetBundleFileType)
}

pub struct AssetBundleLoader {
    bundles: Vec<AssetBundle>
}

impl AssetBundleLoader {
    pub fn get_assets(&self) -> &[AssetBundle] {
        &self.bundles
    }

    pub async fn load_from_directory(path: PathBuf) -> Result<Self, ()> {
        let path = match tokio::fs::canonicalize(&path).await {
            Ok(canonicalized) => canonicalized,
            Err(canon_error) => {
                tracing::error!("Error canonicalizing asset directory path: {} {}", path.to_string_lossy(), canon_error);
                return Err(());
            },
        };
        
        let load_order_file_path = path.join("assets.json");
        let load_order_file = match tokio::fs::read_to_string(&load_order_file_path).await {
            Ok(contents) => contents,
            Err(read_error) => {
                tracing::error!("Error loading asset load order file from path: {} {}", load_order_file_path.to_string_lossy(), read_error);
                return Err(());
            },
        };

        let load_order_file = match serde_json::de::from_str::<LoadOrderFile>(&load_order_file) {
            Ok(parsed) => parsed,
            Err(invalid_format) => {
                tracing::error!("Asset load order file at {} is invalid format: {}", load_order_file_path.to_string_lossy(), invalid_format);
                return Err(());
            },
        };

        let tasks = load_order_file.files.iter().map(|asset_file| (asset_file, &path)).map(|(asset_file, path)| async move {
            let asset_file = match tokio::fs::canonicalize(path.join(asset_file)).await {
                Ok(canonicalized) => canonicalized,
                Err(canon_error) => {
                    tracing::error!("Error canonicalizing asset path: {} {}", asset_file.to_string_lossy(), canon_error);
                    return Err(());
                },
            };

            if asset_file.starts_with(&path) == false {
                tracing::error!("Asset bundle path is outside specified config path: {}", asset_file.to_string_lossy());
                return Err(());
            }

            let metadata = match tokio::fs::metadata(&asset_file).await {
                Ok(metadata) => metadata,
                Err(bad_file) => {
                    tracing::error!("Asset bundle information could not be loaded for: {} with error {}", asset_file.to_string_lossy(), bad_file);
                    return Err(());
                },
            };

            let bundle_type = match metadata.is_file() {
                true => {
                    match asset_file.extension() {
                        Some(extension) => {
                            let file_type = &&*extension.to_string_lossy();
                            match AssetBundleFileType::from_file_extension(&file_type) {
                                Some(valid) => {
                                    AssetBundleType::Zipped(valid)
                                },
                                None => {
                                    tracing::error!("Asset file has an unsupported extension: {}", asset_file.to_string_lossy());
                                    return Err(());
                                },
                            }
                        },
                        None => {
                            tracing::error!("Asset file does not have an extension, which is required: {}", asset_file.to_string_lossy());
                            return Err(());
                        },
                    }
                },
                false => {
                    AssetBundleType::Folder
                },
            };

            let name = match asset_file.file_stem() {
                Some(name) => name.to_string_lossy(),
                None => {
                    tracing::error!("Asset bundle does not have a name, which is required: {}", asset_file.to_string_lossy());
                    return Err(());
                },
            };

            Ok(AssetBundle{bundle_type, name: name.to_string(), path: asset_file})
        });

        let completed = join_all(tasks).await;

        let mut error_loading = false;

        let mut uniqueness_set = HashSet::<String>::new();

        let loaded = completed.into_iter().filter_map(|assets| {
            match assets {
                Ok(valid) => {
                    if uniqueness_set.insert(valid.name.clone()) == false {
                        error_loading = true;
                        tracing::error!("Error loading asset bundles, duplicate name encountered for bundle at: {}", valid.path.to_string_lossy());
                        None
                    } else {
                        Some(valid)
                    }
                },
                Err(()) => {
                    error_loading = true;
                    None
                }
            }
        }).collect();

        match error_loading {
            true => {
                tracing::error!("Error loading asset bundles from file {}", path.to_string_lossy());
                Err(())
            },
            false => {
                Ok(AssetBundleLoader{bundles: loaded})
            },
        }
    }
}
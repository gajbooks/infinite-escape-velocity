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
    path::{Path, PathBuf},
};

use async_zip::tokio::read::fs::ZipFileReader;
use bytes::Bytes;
use futures_util::{future::join_all, AsyncReadExt};
use tokio::task::spawn_blocking;

use crate::configuration_file_structures::asset_definition_file::AssetDefinitionFile;

use super::asset_bundle_loader::AssetBundle;

trait GetFileData {
    async fn get_directories(&self) -> Vec<String>;

    // Even though Zip operates entirely on strings, we need the component parsing for normalizing inconsistent directories for the filesystem implementation
    async fn try_get_file(&self, name: &Path) -> Result<Option<Bytes>, String>;
}

struct ZipReader {
    file: ZipFileReader,
    index: HashMap<String, usize>,
}

impl ZipReader {
    pub async fn new(path: &Path) -> Result<Self, String> {
        match ZipFileReader::new(path).await {
            Ok(file) => {
                let index = file
                    .file()
                    .entries()
                    .iter()
                    .enumerate()
                    .filter_map(
                        |(index, zip_contents)| match zip_contents.filename().as_str() {
                            Ok(valid) => Some((valid.to_string(), index)),
                            Err(not_utf8) => {
                                tracing::error!(
                                    "File name is not UTF 8 in asset bundle zip: {}",
                                    not_utf8.to_string()
                                );
                                None
                            }
                        },
                    )
                    .collect();
                Ok(Self { file: file, index })
            }
            Err(error) => Err(error.to_string()),
        }
    }
}

impl GetFileData for ZipReader {
    async fn get_directories(&self) -> Vec<String> {
        // Verbatim the logic used to determine if a file is a directory per async-zip crate
        self.index
            .iter()
            .filter(|(filename, _index)| filename.ends_with('/'))
            .map(|(filename, _index)| filename)
            .cloned()
            .collect()
    }

    async fn try_get_file(&self, name: &Path) -> Result<Option<Bytes>, String> {
        //We don't have real paths in Zip land
        let name = name.to_string_lossy();
        match self.index.get(&*name) {
            Some(has) => {
                match self.file.reader_with_entry(*has).await {
                    Ok(mut has_entry) => {
                        let mut buf =
                            Vec::with_capacity(has_entry.entry().uncompressed_size() as usize);
                        match has_entry.read_to_end(&mut buf).await {
                            Ok(_completed) => Ok(Some(buf.into())),
                            Err(read_error) => {
                                // File read has failed
                                Err(format!(
                                    "Error reading file {} from zip asset bundle: {}",
                                    name, read_error
                                ))
                            }
                        }
                    }
                    Err(no_reader) => {
                        // Something interesting has happened between filename and index association and an attempted read
                        Err(format!(
                            "Invalid zip index for asset bundle {} with error {}",
                            self.file.path().to_string_lossy(),
                            no_reader.to_string()
                        ))
                    }
                }
            }
            // Zip simply does not have this file, not an error condition
            None => Ok(None),
        }
    }
}

struct FolderReader {
    file: PathBuf,
}

impl FolderReader {
    pub fn new(path: &Path) -> Result<Self, String> {
        match path.canonicalize() {
            Ok(exists) => Ok(Self { file: exists }),
            Err(error) => Err(error.to_string()),
        }
    }
}

impl GetFileData for FolderReader {
    async fn get_directories(&self) -> Vec<String> {
        let base_path = self.file.clone();
        let directories: Vec<_> = spawn_blocking(|| {
            walkdir::WalkDir::new(base_path)
                .into_iter()
                .filter_map(|dir| match dir {
                    Ok(path) => match path.file_type().is_dir() {
                        true => {
                            Some(path.path().join("").to_string_lossy().to_string())
                        }
                        false => None,
                    },
                    Err(error) => {
                        tracing::error!("Error walking directory entry {}", error);
                        None
                    }
                })
                .collect()
        })
        .await
        .unwrap();
        directories
    }

    async fn try_get_file(&self, name: &Path) -> Result<Option<Bytes>, String> {
        let search_path = self.file.join(PathBuf::from(name));
        let canon_path = match tokio::fs::canonicalize(&search_path).await {
            Ok(canon) => canon,
            Err(_error) => {
                // Path does not exist
                return Ok(None);
            }
        };

        if canon_path.starts_with(&self.file) == false {
            return Err(format!(
                "Directory traversal forbidden for path {}",
                canon_path.to_string_lossy()
            ));
        }

        match tokio::fs::read(canon_path).await {
            Ok(read) => Ok(Some(read.into())),
            Err(error) => Err(format!(
                "Error reading asset file from directory: {}",
                error
            )),
        }
    }
}
pub struct AssetFileCache {
    assets: HashMap<String, (AssetDefinitionFile, Bytes)>,
}

impl AssetFileCache {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }

    pub fn get_asset_data(&self, asset_name: &str) -> Option<(AssetDefinitionFile, Bytes)> {
        self.assets.get(asset_name).cloned()
    }

    pub async fn load_asset_bundle(&mut self, file: &AssetBundle) -> Result<(), String> {
        match &file.bundle_type {
            super::asset_bundle_loader::AssetBundleType::Folder => {
                match FolderReader::new(&file.path) {
                    Ok(has) => self.load_asset_bundle_generic(file, has).await,
                    Err(error) => {
                        return Err(error);
                    }
                }
            }
            super::asset_bundle_loader::AssetBundleType::Zipped(zip_type) => match zip_type {
                super::asset_bundle_loader::AssetBundleFileType::Zip => {
                    match ZipReader::new(&file.path).await {
                        Ok(has) => self.load_asset_bundle_generic(file, has).await,
                        Err(error) => {
                            return Err(error);
                        }
                    }
                }
            },
        }
    }

    async fn load_asset_bundle_generic(
        &mut self,
        file: &AssetBundle,
        asset_loader: impl GetFileData,
    ) -> Result<(), String> {
        let file_get_tasks = asset_loader
            .get_directories()
            .await
            .into_iter()
            .map(|possible_directory| (possible_directory, &asset_loader))
            .map(|(possible_directory, asset_loader)| async move {
                let possible_asset_file = PathBuf::from(possible_directory.to_owned()).join("asset.json");
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
                Err(errored) => {
                    tracing::error!("Error reading asset json file: {}", errored);
                    error_reading_asset_json = true;
                    None
                }
            })
            .collect();

        let found_asset_files: Vec<_> = found_asset_files.into_iter().filter_map(|(containing_directory, asset_json_data)| {
            match serde_json::de::from_slice::<AssetDefinitionFile>(&asset_json_data) {
                Ok(deserialized) => Some((containing_directory, deserialized)),
                Err(error_deserializing) => {
                    tracing::error!("Error deserializing asset.json from asset bundle {} with path {} and error {}", file.path.to_string_lossy(), containing_directory, error_deserializing);
                    error_reading_asset_json = true;
                    None
                },
            }
        }).collect();

        if error_reading_asset_json {
            return Err(format!(
                "Error reading all asset.json files from bundle {}",
                file.path.to_string_lossy()
            ));
        }

        let mut duplicate_name_checker = HashSet::<String>::new();

        for (_path, asset_info) in &found_asset_files {
            if duplicate_name_checker.insert(asset_info.asset_name.clone()) == false {
                tracing::warn!("Duplicated asset name found within bundle {} with name {}, this could be an error or result in inconsistent load orders!", file.path.to_string_lossy(), asset_info.asset_name);
            }
        }

        let mut error_reading_asset_files = false;

        let read_asset_file_tasks = found_asset_files
            .into_iter()
            .map(|possible_directory| (possible_directory, &asset_loader))
            .map(
                |((containing_directory, asset_definition), asset_loader)| async move {
                    let name = asset_definition.filename.clone();
                    (
                        asset_definition,
                        asset_loader
                            .try_get_file(&PathBuf::from(&containing_directory).join(&name))
                            .await,
                    )
                },
            );

        let read_asset_files: Vec<_> = join_all(read_asset_file_tasks).await.into_iter().map(|(asset_definition, asset_data)| {
            match asset_data {
                Ok(read) => {
                    match read {
                        Some(has_file) => {
                            Some((asset_definition, has_file))
                        },
                        None => {
                            tracing::error!("Associated asset file does not exist for asset json {}", &asset_definition.filename);
                            error_reading_asset_files = true;
                            None
                        },
                    }
                },
                Err(error_reading) => {
                    tracing::error!("File read error reading asset data from bundle for asset specified as {} with error {}", &asset_definition.filename, error_reading);
                    error_reading_asset_files = true;
                    None
                },
            }
        }).filter_map(|x| x).collect();

        if error_reading_asset_files {
            return Err(format!(
                "Error reading associated asset files for asset.json files from bundle {}",
                file.path.to_string_lossy()
            ));
        }

        for (asset_info, data) in read_asset_files {
            tracing::trace!("Loaded asset {}", asset_info.asset_name);
            self.assets
                .insert(asset_info.asset_name.clone(), (asset_info, data));
        }

        Ok(())
    }
}

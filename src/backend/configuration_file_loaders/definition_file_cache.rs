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

use std::{collections::HashMap, ffi::OsStr, path::PathBuf};

use crate::{
    backend::configuration_file_loaders::{
        asset_bundle_loader::{AssetBundleFileType, AssetBundleType},
        definition_caches::definition_cache::DefinitionCache,
    },
    configuration_file_structures::{
        planetoid_configuration_file::{PlanetoidConfigurationFile, PlanetoidRecord},
        reference_types::{PlanetoidReference, ShipReference},
        ship_configuration_file::{ShipConfigurationFile, ShipRecord},
    },
};

use super::{
    archive_readers::{
        archive_reader::ArchiveReader, filesystem_reader::FilesystemReader, zip_reader::ZipReader,
    },
    asset_bundle_loader::AssetBundle,
    definition_caches::list_required_assets::ListRequiredAssets,
};

enum DefinitionFileNames {
    Planetoids,
    Ships,
}

impl DefinitionFileNames {
    pub fn path_to_definition_type(path: &str) -> Option<DefinitionFileNames> {
        match path {
            "planetoids.json" => Some(DefinitionFileNames::Planetoids),
            "ships.json" => Some(DefinitionFileNames::Ships),
            _ => None,
        }
    }
}

pub struct DefinitionFileCache {
    planetoids: DefinitionCache<PlanetoidRecord, PlanetoidReference>,
    ships: DefinitionCache<ShipRecord, ShipReference>,
}

impl DefinitionFileCache {
    pub fn new() -> DefinitionFileCache {
        DefinitionFileCache {
            planetoids: DefinitionCache::new(),
            ships: DefinitionCache::new(),
        }
    }

    pub fn get_planetoids(&self) -> &[PlanetoidRecord] {
        self.planetoids.get_all_records()
    }

    pub async fn load_definition_bundle(&mut self, file: &AssetBundle) -> Result<(), ()> {
        match &file.bundle_type {
            AssetBundleType::Folder => {
                if let Ok(has) = FilesystemReader::new(&file.path).await {
                    self.load_definition_bundle_generic(file, has).await
                } else {
                    return Err(());
                }
            }
            AssetBundleType::Zipped(zip_type) => match zip_type {
                AssetBundleFileType::Zip => {
                    if let Ok(has) = ZipReader::new(&file.path).await {
                        self.load_definition_bundle_generic(file, has).await
                    } else {
                        return Err(());
                    }
                }
            },
        }
    }

    async fn load_definition_bundle_generic(
        &mut self,
        file: &AssetBundle,
        asset_loader: impl ArchiveReader,
    ) -> Result<(), ()> {
        let mut files_with_extensions = HashMap::<&OsStr, Vec<&PathBuf>>::new();

        let files = asset_loader.get_files().await;

        for definition_file in &files {
            if let Some(named_file) = definition_file.file_name() {
                match files_with_extensions.entry(named_file) {
                    std::collections::hash_map::Entry::Occupied(mut has) => {
                        has.get_mut().push(definition_file);
                    }
                    std::collections::hash_map::Entry::Vacant(empty) => {
                        let has = empty.insert(Vec::new());
                        has.push(definition_file);
                    }
                }
            }
        }

        for file_name in files_with_extensions {
            if let Some(known_type) =
                DefinitionFileNames::path_to_definition_type(&*file_name.0.to_string_lossy())
            {
                for record_asset_file in file_name.1 {
                    if let Ok(no_error) = asset_loader.try_get_file(&record_asset_file).await {
                        if let Some(file_data) = no_error {
                            match known_type {
                                DefinitionFileNames::Planetoids => {
                                    match serde_json::de::from_slice::<PlanetoidConfigurationFile>(
                                        &file_data,
                                    ) {
                                        Ok(deserialized) => {
                                            match self
                                                .planetoids
                                                .add_records(deserialized.definitions.into_iter())
                                            {
                                                Ok(()) => {
                                                    // No problem here
                                                }
                                                Err(()) => {
                                                    tracing::error!(
                                                        "Error loading planetoid file {} from definition bundle {}",
                                                        record_asset_file.to_string_lossy(),
                                                        file.name
                                                    );
                                                    return Err(());
                                                }
                                            }
                                        }
                                        Err(error_deserializing) => {
                                            tracing::error!(
                                                "Error deserializing planetoids.json from definition bundle {} with error {}",
                                                file.path.to_string_lossy(),
                                                error_deserializing
                                            );
                                            return Err(());
                                        }
                                    }
                                }
                                DefinitionFileNames::Ships => {
                                    match serde_json::de::from_slice::<ShipConfigurationFile>(
                                        &file_data,
                                    ) {
                                        Ok(deserialized) => {
                                            match self
                                                .ships
                                                .add_records(deserialized.definitions.into_iter())
                                            {
                                                Ok(()) => {
                                                    // No problem here
                                                }
                                                Err(()) => {
                                                    tracing::error!(
                                                        "Error loading ship file {} from definition bundle {}",
                                                        record_asset_file.to_string_lossy(),
                                                        file.name
                                                    );
                                                    return Err(());
                                                }
                                            }
                                        }
                                        Err(error_deserializing) => {
                                            tracing::error!(
                                                "Error deserializing ships.json from definition bundle {} with error {}",
                                                file.path.to_string_lossy(),
                                                error_deserializing
                                            );
                                            return Err(());
                                        }
                                    }
                                }
                            }
                        } else {
                            tracing::warn!(
                                "File {} from definition bundle {} has suddenly gone missing between directory enumeration and file loading",
                                record_asset_file.to_string_lossy(),
                                file.name
                            );
                        }
                    } else {
                        tracing::error!("Error reading definition bundle {}", file.name);
                        return Err(());
                    }
                }
            } else {
                tracing::warn!(
                    "Unknown file with name {} found in definition bundle {}",
                    file_name.0.to_string_lossy(),
                    file.name
                );
            }
        }

        Ok(())
    }
}

impl ListRequiredAssets for DefinitionFileCache {
    fn get_required_asset_list(
        &self,
    ) -> Vec<(
        &crate::configuration_file_structures::reference_types::AssetReference,
        crate::configuration_file_structures::asset_definition_file::AssetType,
    )> {
        self.planetoids.get_required_asset_list()
    }
}

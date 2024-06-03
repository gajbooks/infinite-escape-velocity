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

use std::collections::HashSet;

use crate::configuration_file_structures::{
    asset_definition_file::AssetType,
    planetoid_configuration_file::PlanetoidRecord,
    reference_types::{AssetReference, PlanetoidReference},
};

use super::list_required_assets::ListRequiredAssets;

pub struct PlanetoidDefinitionCache {
    planetoids: Vec<PlanetoidRecord>,
    planetoid_reference_names: HashSet<PlanetoidReference>,
}

impl PlanetoidDefinitionCache {
    pub fn new() -> PlanetoidDefinitionCache {
        PlanetoidDefinitionCache {
            planetoids: Vec::new(),
            planetoid_reference_names: HashSet::new(),
        }
    }

    pub fn add_planetoid_records(
        &mut self,
        records: impl Iterator<Item = PlanetoidRecord>,
    ) -> Result<(), ()> {
        let mut duplicated_name = false;

        for planetoid in records {
            // Other verification steps may be done here with regards to required definitions for the planetoids

            match self
                .planetoid_reference_names
                .insert(planetoid.planetoid_reference.clone())
            {
                true => {
                    // No problem, name is unique
                }
                false => {
                    tracing::error!(
                        "Duplicated planetoid record name {} found",
                        planetoid.planetoid_reference
                    );
                    duplicated_name = true;
                }
            };

            if !duplicated_name {
                tracing::trace!(
                    "Loaded planetoid definition {}",
                    planetoid.planetoid_reference
                );
                self.planetoids.push(planetoid);
            }
        }

        if duplicated_name {
            tracing::error!("Bundle could not be loaded due to duplicate names from self or previously loaded bundles");
            return Err(());
        } else {
            return Ok(());
        }
    }

    pub fn get_all_planetoid_records(&self) -> &[PlanetoidRecord] {
        return &self.planetoids;
    }
}

impl ListRequiredAssets for PlanetoidDefinitionCache {
    fn get_required_asset_list(&self) -> Vec<(&AssetReference, AssetType)> {
        self.planetoids
            .iter()
            .flat_map(|record| record.get_required_asset_list())
            .collect()
    }
}

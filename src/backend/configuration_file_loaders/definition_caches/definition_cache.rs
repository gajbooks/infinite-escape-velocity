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

use crate::{
    backend::configuration_file_loaders::definition_caches::definition_cachable::DefinitionCachable,
    configuration_file_structures::{
        asset_definition_file::AssetType, reference_types::AssetReference,
    },
};

use super::list_required_assets::ListRequiredAssets;

pub struct DefinitionCache<T: DefinitionCachable<U>, U: Eq + std::hash::Hash + std::fmt::Display> {
    items: Vec<T>,
    reference_names: HashSet<U>,
}

impl<T: DefinitionCachable<U>, U: Eq + std::hash::Hash + std::fmt::Display> DefinitionCache<T, U> {
    pub fn new() -> DefinitionCache<T, U> {
        Self {
            items: Vec::new(),
            reference_names: HashSet::new(),
        }
    }

    pub fn add_records(&mut self, records: impl Iterator<Item = T>) -> Result<(), ()> {
        let mut duplicated_name = false;

        for item in records {
            // Other verification steps may be done here with regards to required definitions for the planetoids

            match self.reference_names.insert(item.get_reference_id()) {
                true => {
                    // No problem, name is unique
                }
                false => {
                    tracing::error!(
                        "Duplicated definition record name {} found",
                        item.get_reference_id()
                    );
                    duplicated_name = true;
                }
            };

            if !duplicated_name {
                tracing::trace!("Loaded definition {}", item.get_reference_id());
                self.items.push(item);
            }
        }

        if duplicated_name {
            tracing::error!(
                "Bundle could not be loaded due to duplicate names from self or previously loaded bundles"
            );
            return Err(());
        } else {
            return Ok(());
        }
    }

    pub fn get_all_records(&self) -> &[T] {
        return &self.items;
    }
}

impl<T: DefinitionCachable<U>, U: Eq + std::hash::Hash + std::fmt::Display> ListRequiredAssets
    for DefinitionCache<T, U>
{
    fn get_required_asset_list(&self) -> Vec<(&AssetReference, AssetType)> {
        self.items
            .iter()
            .flat_map(|record| record.get_required_asset_list())
            .collect()
    }
}

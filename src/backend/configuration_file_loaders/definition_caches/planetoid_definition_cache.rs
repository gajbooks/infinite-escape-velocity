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

use crate::configuration_file_structures::planetoid_configuration_file::PlanetoidRecord;

pub struct PlanetoidDefinitionCache {
    planetoids: Vec<PlanetoidRecord>,
    planetoid_names: HashSet::<String>
}

impl PlanetoidDefinitionCache {
    pub fn new() -> PlanetoidDefinitionCache {
        PlanetoidDefinitionCache{planetoids: Vec::new(), planetoid_names: HashSet::new()}
    }

    pub fn add_planetoid_records(&mut self, records: impl Iterator<Item = PlanetoidRecord>) -> Result<(), ()> {

        for planetoid in records {
            // Other verification steps may be done here with regards to required definitions for the planetoids

            match self.planetoid_names.insert(planetoid.planetoid_name.clone()) {
                true => {
                    // No problem, name is unique
                },
                false => {
                    tracing::error!("Duplicated planetoid name {} found", planetoid.planetoid_name);
                    return Err(());
                }
            };

            self.planetoids.push(planetoid);
        }

        Ok(())
    }
}
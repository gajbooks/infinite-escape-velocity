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

use fxhash::FxBuildHasher;
use dashmap::DashMap;
use super::object_configuration_record::*;
use super::object_configuration_file::*;
use std::fs::File;
use std::path::Path;

pub struct ObjectConfigurationMap {
    object_prototypes: DashMap<ObjectTypeParameters, ObjectConfigurationRecord, FxBuildHasher>
}

impl ObjectConfigurationMap {
    pub fn from_file(base_directory: &Path, filename: &Path) -> Result<ObjectConfigurationMap, ()> {
        let configuration_file = match File::open(base_directory.join(filename)) {
            Ok(opened) => opened,
            Err(error) => {
                println!("Error opening dynamic object configuration file: {}", error);
                return Err(());
            }
        };

        let parsed_data: ObjectConfigurationFile = match serde_json::from_reader(configuration_file) {
            Ok(parsed) => parsed,
            Err(error) => {
                println!("Parsing the dynamic object configuration file has an error: {}", error);
                return Err(());
            }
        };

        let verify_list = DashMap::with_hasher(FxBuildHasher::default());

        for record in parsed_data.definitions {
            match verify_list.entry(record.object_type.clone()) {
                dashmap::mapref::entry::Entry::Occupied(has) => {
                    println!("Duplicate type record found in dynamic object configuration file: {:?}", has.key());
                    return Err(());
                }
                dashmap::mapref::entry::Entry::Vacant(empty) => {
                    empty.insert(record);
                }
            }
        }

        return Ok(
            ObjectConfigurationMap {
                object_prototypes: verify_list
            }
        );
    }

    pub fn get(&self, object_type: &ObjectTypeParameters) -> Option<ObjectConfigurationRecord> {
        match self.object_prototypes.get(object_type) {
            Some(has) => Some(has.value().clone()),
            None => None
        }
    }

    pub fn get_by_components(&self, author: &str, object_type: &str) -> Option<ObjectConfigurationRecord> {
        return self.get(&ObjectTypeParameters{author: author.to_owned(), object_type: object_type.to_owned()});
    }

    pub fn get_all(&self) -> Vec<ObjectConfigurationRecord> {
        return self.object_prototypes.iter().map(|x| x.value().clone()).collect();
    }
}
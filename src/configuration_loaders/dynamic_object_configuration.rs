use fxhash::FxBuildHasher;
use dashmap::DashMap;
use super::dynamic_object_record::*;
use super::dynamic_object_file::*;
use std::fs::File;
use std::path::Path;

pub struct DynamicObjectConfiguration {
    object_prototypes: DashMap<DynamicObjectTypeParameters, DynamicObjectRecord, FxBuildHasher>
}

impl DynamicObjectConfiguration {
    pub fn from_file(base_directory: &Path, filename: &Path) -> Result<DynamicObjectConfiguration, ()> {
        let configuration_file = match File::open(base_directory.join(filename)) {
            Ok(opened) => opened,
            Err(error) => {
                println!("Error opening dynamic object configuration file: {}", error);
                return Err(());
            }
        };

        let parsed_data: DynamicObjectFile = match serde_json::from_reader(configuration_file) {
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
            DynamicObjectConfiguration {
                object_prototypes: verify_list
            }
        );
    }

    pub fn get(&self, object_type: &DynamicObjectTypeParameters) -> Option<DynamicObjectRecord> {
        match self.object_prototypes.get(object_type) {
            Some(has) => Some(has.value().clone()),
            None => None
        }
    }

    pub fn get_by_components(&self, author: &str, object_type: &str) -> Option<DynamicObjectRecord> {
        return self.get(&DynamicObjectTypeParameters{author: author.to_owned(), object_type: object_type.to_owned()});
    }

    pub fn get_all(&self) -> Vec<DynamicObjectRecord> {
        return self.object_prototypes.iter().map(|x| x.value().clone()).collect();
    }
}
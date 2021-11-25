use super::dynamic_object_record::*;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct DynamicObjectFile {
    pub version: u32,
    pub definitions: Vec<DynamicObjectRecord>
}
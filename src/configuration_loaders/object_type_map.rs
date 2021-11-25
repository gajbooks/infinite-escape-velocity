use super::super::shared_types::*;
use dashmap::DashMap;
use fxhash::FxBuildHasher;
use super::dynamic_object_record::*;

pub struct ObjectTypeMap {
    text_to_type: DashMap<DynamicObjectTypeParameters, ObjectType, FxBuildHasher>,
    type_to_text: DashMap<ObjectType, DynamicObjectTypeParameters, FxBuildHasher>,
    current_object_id: AtomicIdType
}

impl ObjectTypeMap {
    pub fn new() -> ObjectTypeMap {
        return ObjectTypeMap {
            text_to_type: DashMap::with_hasher(FxBuildHasher::default()),
            type_to_text: DashMap::with_hasher(FxBuildHasher::default()),
            current_object_id: AtomicIdType::new(0)};
    }

    pub fn object_type_parameters_to_object_type(&self, object_type: &DynamicObjectTypeParameters) -> Result<ObjectType, ()> {
        return match self.text_to_type.get(object_type) {
            Some(has) => Ok(has.value().clone()),
            None => Err(())
        }
    }

    pub fn object_type_to_object_type_parameters(&self, object_type: &ObjectType) -> Result<Option<DynamicObjectTypeParameters>, ()> {
        match object_type {
            ObjectType::WorldObject(_type_data) => {
                match self.type_to_text.get(object_type) {
                    Some(has) => Ok(Some(has.value().clone())),
                    None => Err(())
                }
            },
            _ => Ok(None)
        }
    }

    pub fn add_object_type(&self, object_type: &DynamicObjectTypeParameters) -> ObjectType {
        match self.object_type_parameters_to_object_type(object_type) {
            Ok(has) => {
                println!("Attempted to register duplicate object type with author {} and type {}", object_type.author, object_type.object_type);
                return has;
            },
            Err(()) => ()
        };

        let new_id = self.current_object_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.text_to_type.insert(object_type.to_owned(), ObjectType::WorldObject(new_id));
        self.type_to_text.insert(ObjectType::WorldObject(new_id), object_type.to_owned());

        return ObjectType::WorldObject(new_id);
    }
}
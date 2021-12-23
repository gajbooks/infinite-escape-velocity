use super::world_interaction_event::*;
use std::sync::*;
use super::unique_object_storage::*;
use super::world_object_constructor::*;
use super::ship::*;

pub struct WorldInteractionHandler {
    storage: Arc<UniqueObjectStorage>,
    world_object_constructor: Arc<WorldObjectConstructor>
}

impl WorldInteractionHandler {
    pub fn new(storage: Arc<UniqueObjectStorage>, world_object_constructor: Arc<WorldObjectConstructor>) -> WorldInteractionHandler {
        return WorldInteractionHandler{storage: storage, world_object_constructor: world_object_constructor};
    }

    pub fn process_set(&self, events: Vec<WorldInteractionEvent>) {
        for event in &events {
            if let WorldInteractionEvent::SpawnObject(spawn) = event {
                let constructed = match self.world_object_constructor.construct_from_type::<Ship>(&spawn.object_type, spawn.position.clone()) {
                    Some(has) => has,
                    None => {
                        println!("Could not construct object for world: {:?}", &spawn.object_type);
                        continue;
                    }
                };

                self.storage.add(constructed);
            }
        }

        for event in &events {
            if let WorldInteractionEvent::RemoveObject(remove) = event {
                match self.storage.remove(*remove) {
                    false => {
                        println!("Tried to remove non-existent object from world with ID: {}", remove);
                    }
                    _ => ()
                }
            }
        }
    }
}
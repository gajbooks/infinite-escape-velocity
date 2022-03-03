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

use super::world_interaction_event::*;
use std::sync::*;
use super::unique_object_storage::unique_object_storage::*;
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
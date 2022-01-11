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

mod backend;
mod configuration_loaders;
mod connectivity;
mod frontend;
mod shared_types;
use backend::player_object_binding::*;
use backend::server_viewport::*;
use backend::shape::*;
use backend::ship::*;
use backend::spatial_hashmap;
use backend::unique_id_allocator::*;
use backend::unique_object_storage::*;
use backend::world_object_constructor::*;
use configuration_loaders::{
    dynamic_object_configuration::*, dynamic_object_record::*, object_type_map::*,
};
use connectivity::client_server_message::*;
use connectivity::server_client_message::*;
use crossbeam_channel::unbounded;
use crossbeam_channel::*;
use frontend::{
    controlled_object_handler::*, frontend_viewport::*, object_texture_mapping::*,
    texture_mapper::*,
};
use macroquad::prelude::*;
use shared_types::*;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::sync::*;

const DEFAULT_OBJECT_FILE: &str = "objects.json";
const DEFAULT_TEXTURE: &str = "images/default.webp";

async fn find_base_data_directory() -> Option<PathBuf> {
    let file = File::open("data-directory.txt");

    match file {
        Ok(has) => {
            let lines = io::BufReader::new(has).lines();
            for line in lines {
                match line {
                    Ok(has_line) => {
                        let path = Path::new(&has_line);
                        match path.is_dir() {
                            true => {
                                return Some(path.to_path_buf());
                            }
                            false => {
                                return None;
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error reading line from data-directory.txt: {}", e);
                        return None;
                    }
                }
            }

            None
        }
        Err(e) => {
            println!(
                "Error opening data-directory.txt to locate base path: {}",
                e
            );
            return None;
        }
    }
}

async fn initialize_client_data(
    client_sender: Sender<ClientServerMessage>,
    server_receiver: Receiver<ServerClientMessage>,
) {
    let base_directory = find_base_data_directory().await.unwrap();

    let object_file_path = Path::new(DEFAULT_OBJECT_FILE);

    let dynamic_object_configuration =
        match DynamicObjectConfiguration::from_file(&base_directory, &object_file_path) {
            Ok(loaded) => Arc::new(loaded),
            Err(()) => {
                panic!("Could not load object definition file");
            }
        };
    
    let default_texture_path = Path::new(DEFAULT_TEXTURE);
    let texture_mapper = Arc::new(TextureMapper::new(&base_directory));
    let default_texture = texture_mapper.load_texture(&default_texture_path).unwrap();

    let type_mapper = Arc::new(ObjectTypeMap::new());

    for i in dynamic_object_configuration.get_all() {
        type_mapper.add_object_type(&i.object_type);

        match &i.graphics_parameters {
            Some(graphics) => {
                match &graphics.simple {
                    Some(simple_texture) => {
                        let texture_path = Path::new(&simple_texture.filename);
                        match texture_mapper.load_texture(&texture_path) {
                            Ok(_loaded) => (),
                            Err(()) => {
                                println!("Failed loading texture {} from path {:?}", &simple_texture.filename, texture_path);
                            }
                        }
                    },
                    None => ()
                }
            },
            None => ()
        };
    }

    TextureMapper::atlasize_textures();

    let object_index = ObjectToTextureIndex::new(
        texture_mapper,
        type_mapper,
        dynamic_object_configuration,
        default_texture,
    );
    let mut viewport = FrontendViewport::new(server_receiver, object_index);
    let mut client_controlled_object_handler = ControlledObjectHandler::new(client_sender);

    let mut viewport_timestamp = std::time::Instant::now();

    loop {
        let new_now = std::time::Instant::now();
        let duration = new_now.duration_since(viewport_timestamp);
        viewport_timestamp = new_now;
        viewport.tick(duration.as_secs_f32()).await;
        client_controlled_object_handler.send_updates();
        next_frame();
    }
}

async fn initialize_server_data(
    client_receiver: Receiver<ClientServerMessage>,
    server_sender: Sender<ServerClientMessage>,
) -> std::thread::JoinHandle<()> {
    let base_directory = find_base_data_directory().await.unwrap();

    let object_file_path = Path::new(DEFAULT_OBJECT_FILE);

    let map = spatial_hashmap::SpatialHashmap::new();
    let storage = Arc::new(UniqueObjectStorage::new());
    let unique_id_generator = Arc::new(UniqueIdAllocator::new());

    let dynamic_object_configuration =
        match DynamicObjectConfiguration::from_file(&base_directory, &object_file_path) {
            Ok(loaded) => Arc::new(loaded),
            Err(()) => {
                panic!("Could not load object definition file");
            }
        };

    let type_mapper = Arc::new(ObjectTypeMap::new());

    for i in dynamic_object_configuration.get_all() {
        type_mapper.add_object_type(&i.object_type);
    }

    let default_type = DynamicObjectTypeParameters {
        author: "default".to_string(),
        object_type: "default".to_string(),
    };

    let world_object_constructor = Arc::new(WorldObjectConstructor::new(
        type_mapper.clone(),
        dynamic_object_configuration.clone(),
        unique_id_generator.clone(),
    ));

    storage.add(
        world_object_constructor
            .construct_from_type::<Ship>(
                &default_type,
                CoordinatesRotation {
                    location: Coordinates::new(50.0, 0.0),
                    rotation: Rotation::radians(0.0),
                },
            )
            .unwrap(),
    );

    storage.add(
        world_object_constructor
            .construct_from_type::<Ship>(
                &default_type,
                CoordinatesRotation {
                    location: Coordinates::new(0.0, 0.0),
                    rotation: Rotation::radians(0.0),
                },
            )
            .unwrap(),
    );
    let viewport_id = unique_id_generator.new_allocated_id();

    let player_object = PlayerObjectBinding::new(
        client_receiver,
        server_sender.clone(),
        storage.clone(),
        viewport_id.id,
    );

    storage.add(Arc::new(ServerViewport::new(
        Shape::Circle(CircleData {
            location: Coordinates::new(0.0, 0.0),
            radius: Radius::new(10000.0),
        }),
        viewport_id,
        server_sender,
        storage.clone(),
    )));

    std::thread::spawn(move || {
        server_loop(storage, player_object, map, world_object_constructor);
    })
}

fn server_loop(storage: Arc<UniqueObjectStorage>, mut player_object: PlayerObjectBinding, map: spatial_hashmap::SpatialHashmap, constructor: Arc<WorldObjectConstructor>) {
    let intraction_handler = backend::world_interaction_handler::WorldInteractionHandler::new(storage.clone(), constructor.clone());
    let physics_update_rate: u64 = 20;

    let physics_tick_duration = 1.0 / physics_update_rate as f32;

    let mut engine_timestamp = std::time::Instant::now();

    loop {
        let engine_now = std::time::Instant::now();
        let engine_duration = engine_now.duration_since(engine_timestamp);
        if engine_duration > std::time::Duration::from_secs_f32(physics_tick_duration) {
            let objects = storage.all_objects();
            let events: Vec<_> = objects
                .iter()
                .map(|x| x.tick(DeltaT::new(physics_tick_duration)))
                .flatten()
                .collect();
            intraction_handler.process_set(events);
            player_object.handle_updates(DeltaT::new(physics_tick_duration));
            map.run_collisions(objects.as_slice());
            engine_timestamp = engine_now;
        } else {
            std::thread::sleep(std::time::Duration::from_secs_f32(
                physics_tick_duration / 2.0,
            ));
        }
    }
}

#[macroquad::main("Infinite Escape Velocity")]
async fn main() {
    let (server_sender, server_receiver) = unbounded();
    let (client_sender, client_receiver) = unbounded();

    let physics_thread = initialize_server_data(client_receiver, server_sender).await;

    initialize_client_data(client_sender, server_receiver).await;

    physics_thread.join().unwrap();
}

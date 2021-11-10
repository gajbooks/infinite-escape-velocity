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
mod frontend;
mod connectivity;
mod shared_types;
use shared_types::*;
use backend::shape::*;
use std::sync::*;
use backend::unique_id_allocator::*;
use backend::unique_object_storage::*;
use backend::player_object_binding::*;
use rayon::prelude::*;
use crossbeam_channel::unbounded;
use macroquad::prelude::*;
use backend::server_viewport::*;
use backend::spatial_hashmap;
use backend::ship;
use frontend::{frontend_viewport::*, texture_mapper::*, object_texture_mapping::*, controlled_object_handler::*};

#[macroquad::main("Infinite Escape Velocity")]
async fn main() {
    let map = spatial_hashmap::SpatialHashmap::new();
    let storage = Arc::new(UniqueObjectStorage::new());
    let unique_id_generator = UniqueIdAllocator::new();

    let texture_mapper = TextureMapper::new();
    texture_mapper.load_texture("default", "../starbridge.webp").unwrap();
    TextureMapper::atlasize_textures();

    let object_index = ObjectIndex::new(texture_mapper);

    let(server_sender, server_receiver) = unbounded();
    let(client_sender, client_receiver) = unbounded();

    let mut client_controlled_object_handler = ControlledObjectHandler::new(client_sender);
    let mut player_object = PlayerObjectBinding::new(client_receiver, server_sender.clone(), storage.clone());

    storage.add(Arc::new(ship::Ship::new(&CoordinatesRotation{x: 0.0, y: 0.0, r: 1.0}, unique_id_generator.new_allocated_id())));
    storage.add(Arc::new(ServerViewport::new(Shape::Circle(CircleData{x: 0.0, y: 0.0, r: 1000.0}), unique_id_generator.new_allocated_id(), server_sender, storage.clone())));

    let mut viewport = FrontendViewport::new(server_receiver, object_index);

    let physics_update_rate: u64 = 60;

    let physics_thread = std::thread::spawn(move || {
        let mut engine_timestamp = std::time::Instant::now();

        loop {
            let objects = storage.all_objects();
            map.run_collisions(objects.as_slice());
            let engine_now = std::time::Instant::now();
            let engine_duration = engine_now.duration_since(engine_timestamp);
            if engine_duration > std::time::Duration::from_secs_f32(1.0/physics_update_rate as f32) {
                player_object.handle_updates(engine_duration.as_secs_f32());
                objects.par_iter().for_each(|x| x.tick(engine_duration.as_secs_f32()));
                engine_timestamp = engine_now;
            } else {
                std::thread::sleep(std::time::Duration::from_secs_f32(0.5/physics_update_rate as f32));
            }
        }
    });

    let maximum_framerate: u64 = 120;

        let mut viewport_timestamp = std::time::Instant::now();

        loop {

            let new_now = std::time::Instant::now();
            let duration = new_now.duration_since(viewport_timestamp);
            if duration > std::time::Duration::from_secs_f32(1.0/maximum_framerate as f32) {
            viewport_timestamp = new_now;
            viewport.tick(duration.as_secs_f32()).await;
            client_controlled_object_handler.send_updates();
            } else {
                std::thread::sleep(std::time::Duration::from_secs_f32(0.5/maximum_framerate as f32));
            }

        }

        physics_thread.join().unwrap();


}

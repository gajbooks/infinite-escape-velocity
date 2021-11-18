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

    let viewport_id = unique_id_generator.new_allocated_id();

    let mut client_controlled_object_handler = ControlledObjectHandler::new(client_sender);
    let mut player_object = PlayerObjectBinding::new(client_receiver, server_sender.clone(), storage.clone(), viewport_id.id);

    storage.add(Arc::new(ServerViewport::new(Shape::Circle(CircleData{location: Coordinates::new(0.0, 0.0), radius: Radius::new(10000.0)}), viewport_id, server_sender, storage.clone())));

    storage.add(Arc::new(ship::Ship::new(CoordinatesRotation{location: Coordinates::new(50.0, 0.0), rotation: Rotation::radians(0.0)}, unique_id_generator.new_allocated_id())));
    storage.add(Arc::new(ship::Ship::new(CoordinatesRotation{location: Coordinates::new(0.0, 0.0), rotation: Rotation::radians(0.0)}, unique_id_generator.new_allocated_id())));

    let mut viewport = FrontendViewport::new(server_receiver, object_index);

    let physics_update_rate: u64 = 20;

    let physics_tick_duration = 1.0 / physics_update_rate as f32;

    let physics_thread = std::thread::spawn(move || {
        let mut engine_timestamp = std::time::Instant::now();

        loop {
            let engine_now = std::time::Instant::now();
            let engine_duration = engine_now.duration_since(engine_timestamp);
            if engine_duration > std::time::Duration::from_secs_f32(physics_tick_duration) {

                let objects = storage.all_objects();
                objects.iter().for_each(|x| x.tick(DeltaT::new(physics_tick_duration)));
                player_object.handle_updates(DeltaT::new(physics_tick_duration));
                map.run_collisions(objects.as_slice());
                engine_timestamp = engine_now;
            } else {
                std::thread::sleep(std::time::Duration::from_secs_f32(physics_tick_duration / 2.0));
            }
        }
    });

        let mut viewport_timestamp = std::time::Instant::now();

        loop {
            let new_now = std::time::Instant::now();
            let duration = new_now.duration_since(viewport_timestamp);
            viewport_timestamp = new_now;
            viewport.tick(duration.as_secs_f32()).await;
            client_controlled_object_handler.send_updates();
            next_frame();
        }

        physics_thread.join().unwrap();


}

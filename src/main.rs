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
mod shared_types;

use crate::backend::shape::Shape;
use axum::{routing::get, Router};
use backend::shape::CircleData;
use backend::spatial_optimizer::collision_optimizer::CollisionOptimizer;
use backend::world_object_storage::ephemeral_id_allocator::{
    EphemeralIdAllocator, IdAllocatorType,
};
use backend::world_object_storage::world_object_storage::WorldObjectStorage;
use backend::world_objects::ship::Ship;
use clap::Parser;
use connectivity::connected_users::ConnectedUsers;
use euclid::{Length, Scale};
use rand::Rng;
use tokio::time;
use tower_http::compression::CompressionLayer;
use tracing::{info, trace, Level};
use tracing_subscriber::FmtSubscriber;

use connectivity::websocket_handler::*;
use euclid::Point2D;
use std::net::SocketAddr;
use std::sync::*;
use std::time::Duration;

use tower_http::services::ServeDir;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Directory to host the webapp from. If ommitted, server is started without.
    #[arg(long)]
    webapp_directory: Option<String>,

    /// Directory to load gamedata from.
    data_directory: String,

    /// Display more in-depth logs
    #[clap(long, action)]
    verbose_logs: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let tracing = match args.verbose_logs {
        true => FmtSubscriber::builder().with_max_level(Level::TRACE),
        false => FmtSubscriber::builder().with_max_level(Level::INFO),
    };

    tracing::subscriber::set_global_default(tracing.finish())
        .expect("Failed to initialize trace logging");

    let app = Router::new();

    let app = match &args.webapp_directory {
        Some(webapp_directory) => app.nest_service("/", ServeDir::new(webapp_directory)),
        None => app,
    };

    let user_connections = Arc::new(ConnectedUsers::new());
    let tick_connections = user_connections.clone();
    let world_object_storage = Arc::new(WorldObjectStorage::new());
    let spawning_storage = world_object_storage.clone();
    let id_allocator: IdAllocatorType = Arc::new(EphemeralIdAllocator::new());
    let spawn_allocator = id_allocator.clone();

    tokio::spawn(ConnectedUsers::garbage_collector(user_connections.clone()));

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(4));

        loop {
            let time = interval.tick().await;
            let x: f64 = rand::thread_rng().gen_range(-100.0..100.0);
            let y: f64 = rand::thread_rng().gen_range(-100.0..100.0);
            spawning_storage.add(Arc::new(Ship::new(
                Shape::Circle(CircleData {
                    location: Point2D::new(x, y),
                    radius: Length::new(5.0 as f64),
                }),
                "A ship I guess".to_string(),
                spawn_allocator.new_id(),
            )));
            info!("Spawned a ship at {:?}", time);
        }
    });

    tokio::spawn(async move {
        let expected_tick_rate = time::Duration::from_secs_f32(1.0 / 20.0);
        let mut collision_optimizer = CollisionOptimizer::new();

        let mut start;
        loop {
            start = time::Instant::now();
            let users = tick_connections.all_users();
            let viewports = users.iter().map(|x| x.get_viewport());
            let mut all_objects = world_object_storage.all_objects();
            all_objects.extend(viewports);

            collision_optimizer.run_collisions(all_objects.as_slice());
            for i in &all_objects {
                i.tick(Scale::new(expected_tick_rate.as_secs_f32()));
            }
            world_object_storage.cleanup();
            let wall_time = time::Instant::now() - start;
            let wait = expected_tick_rate - wall_time;
            time::sleep(wait).await;
        }
    });

    let websocket_state = HandlerState {
        connections: user_connections.clone(),
        id_allocator: id_allocator,
    };

    let app = app
        .route("/ws", get(websocket_handler))
        .with_state(websocket_state)
        .layer(CompressionLayer::new());

    axum::Server::bind(&"0.0.0.0:2718".parse().unwrap())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

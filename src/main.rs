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

use axum::{routing::get, Router};
use backend::resources::delta_t_resource::{increment_time, DeltaTResource};
use backend::shape::{Shape, PointData};
use backend::spatial_optimizer::collision_optimizer::{CollisionOptimizer, collision_system};
use backend::world_objects::object_properties::collision_component::{clear_old_collisions, CollisionMarker};
use backend::world_objects::object_properties::timeout_component::{check_despawn_times, TimeoutComponent};
use backend::world_objects::server_viewport::{tick_viewport, Displayable};
use backend::world_objects::ship::ShipBundle;
use bevy_ecs::schedule::{Schedule, IntoSystemConfigs};
use bevy_ecs::system::{Local, Commands, Res};
use bevy_ecs::world::World;
use clap::Parser;
use connectivity::connected_users::{ConnectedUsers, create_user_viewports, ConnectedUsersResource};
use shared_types::Coordinates;
use tokio::time;
use tower_http::compression::CompressionLayer;
use tracing::{trace, Level};
use tracing_subscriber::FmtSubscriber;

use connectivity::websocket_handler::*;
use std::net::SocketAddr;
use std::sync::*;
use std::time::Duration;

use tower_http::services::ServeDir;

use crate::backend::resources::delta_t_resource::MINIMUM_TICK_DURATION;

fn spawn_a_ship_idk(mut spawned: Local<u32>, time: Res<DeltaTResource>, mut commands: Commands) {
    if *spawned == 0 {
        *spawned = 1;
    }

    if (time.total_time / *spawned) > Duration::from_secs(1) {
        *spawned += 1;
        commands.spawn(ShipBundle {
            displayable: Displayable{object_type: format!("Ship {}", *spawned)},
            displayable_collision_marker: CollisionMarker::<Displayable>::new(Shape::Point(PointData{point: Coordinates::new(0.0, 0.0)})),
            timeout: TimeoutComponent{spawn_time: time.total_time, lifetime: Duration::from_secs(2)}
        });
    }
}

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
    let resource_connections = user_connections.clone();

    tokio::spawn(ConnectedUsers::garbage_collector(user_connections.clone()));

    tokio::spawn(async move {
        let mut world = World::new();
        world.insert_resource(DeltaTResource::new());
        world.insert_resource(CollisionOptimizer::<Displayable>::new());
        world.insert_resource(ConnectedUsersResource{connected_users: resource_connections});

        let mut schedule = Schedule::default();
        schedule.add_systems(increment_time);
        schedule.add_systems(clear_old_collisions::<Displayable>);
        schedule.add_systems(collision_system::<Displayable>.after(clear_old_collisions::<Displayable>));
        schedule.add_systems(create_user_viewports);
        schedule.add_systems(tick_viewport.after(collision_system::<Displayable>));
        schedule.add_systems(spawn_a_ship_idk.after(increment_time));
        schedule.add_systems(check_despawn_times.after(increment_time));

        const STATS_INTERVAL: usize = 1000;
        let mut stats_counter: usize = 0;
        let mut average_time: f32 = 0.0;
        loop {
            let now = time::Instant::now();
            schedule.run(&mut world);
            let duration = time::Instant::now().duration_since(now);
            {
                let mut world_tick = world.get_resource_mut::<DeltaTResource>().unwrap();
                world_tick.last_tick_time = duration;
            }
            stats_counter += 1;
            average_time += duration.as_secs_f32() / 1000.0;
            if stats_counter == STATS_INTERVAL {
                trace!("Ticked in {} milliseconds average with {} entities currently", average_time / STATS_INTERVAL as f32, world.entities().len());
                stats_counter = 0;
            }
            let minimum_time = MINIMUM_TICK_DURATION.saturating_sub(duration);
            let time_less_milliseconds = minimum_time.saturating_sub(Duration::from_millis(2));
            time::sleep(time_less_milliseconds).await;
            let true_time_now = time::Instant::now();
            let time_remainder = minimum_time.saturating_sub(true_time_now - now);
            spin_sleep::sleep(time_remainder);
        }
    });

    let websocket_state = HandlerState {
        connections: user_connections.clone(),
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

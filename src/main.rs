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
mod configuration_file_structures;
mod connectivity;
mod shared_types;

use axum::{routing::get, Router};
use backend::configuration_file_loaders::asset_bundle_loader::AssetBundleLoader;
use backend::resources::delta_t_resource::{increment_time, DeltaTResource};
use backend::spatial_optimizer::collision_optimizer::{collision_system, CollisionOptimizer};
use backend::spatial_optimizer::hash_sized::HashSized;
use backend::world_objects::components::collision_component::clear_old_collisions;
use backend::world_objects::components::semi_newtonian_physics_component::SemiNewtonianPhysicsComponent;
use backend::world_objects::components::timeout_component::{
    check_despawn_times, TimeoutComponent,
};
use backend::world_objects::server_viewport::{tick_viewport, Displayable};
use backend::world_objects::ship::ShipBundle;
use bevy_ecs::schedule::{IntoSystemConfigs, Schedule};
use bevy_ecs::system::{Commands, Local, Res};
use bevy_ecs::world::World;
use clap::Parser;
use connectivity::connected_users::ConnectingUsersQueue;
use euclid::Angle;
use futures::channel::mpsc::unbounded;
use rand::Rng;
use shared_types::{Coordinates, Speed, Velocity};
use tokio::time;
use tower_http::compression::CompressionLayer;
use tracing::{debug, trace, Level};
use tracing_subscriber::FmtSubscriber;

use connectivity::websocket_handler::*;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::layer::SubscriberExt;

use tower_http::services::ServeDir;

use crate::backend::configuration_file_loaders::asset_file_cache::AssetFileCache;
use crate::backend::configuration_file_loaders::definition_file_cache::DefinitionFileCache;
use crate::backend::resources::delta_t_resource::MINIMUM_TICK_DURATION;
use crate::backend::systems::apply_player_control::apply_player_control;
use crate::backend::systems::player_spawn_system::spawn_player_ship_and_viewports;
use crate::backend::systems::update_collisions_with_position::update_collisions_with_position;
use crate::backend::systems::update_collisions_with_rotation::update_collisions_with_rotation;
use crate::backend::systems::update_positions_with_velocity::update_positions_with_velocity;
use crate::backend::systems::update_rotations_with_angular_velocity::update_rotations_with_angular_velocity;
use crate::backend::systems::update_velocities_with_semi_newtonian_physics::update_velocities_with_semi_newtonian_physics;
use crate::connectivity::asset_server::{ asset_by_name, AssetServerState};
use crate::connectivity::connected_users::{check_alive_sessions, spawn_user_sessions};
use crate::connectivity::user_session::{process_incoming_messages, UserSession};

fn plus_or_minus_random(radius: f64) -> f64 {
    let value = rand::thread_rng().gen::<f64>();
    let range = radius * 2.0;
    (range * value) - radius
}

fn spawn_a_ship_idk(mut spawned: Local<u32>, time: Res<DeltaTResource>, mut commands: Commands) {
    if *spawned == 0 {
        *spawned = 1;
    }

    if (time.total_time / *spawned) > Duration::from_secs(1) {
        *spawned += 1;
        commands.spawn((
            ShipBundle::new(
                &spawned.to_string(),
                Coordinates::new(plus_or_minus_random(100.0), plus_or_minus_random(100.0)),
                Some(Velocity::new(
                    plus_or_minus_random(100.0) as f32,
                    plus_or_minus_random(100.0) as f32,
                )),
                Some(Angle::radians(
                    plus_or_minus_random(std::f64::consts::PI) as f32
                )),
                Some(Angle::radians(
                    plus_or_minus_random(std::f64::consts::PI) as f32
                )),
            ),
            SemiNewtonianPhysicsComponent::new(Speed::new(50.0)),
            TimeoutComponent {
                spawn_time: time.total_time,
                lifetime: Duration::from_secs(10),
            },
        ));
    }
}

fn build_collision_phase<T: Send + Sync + HashSized + 'static>(
    schedule: &mut Schedule,
    world: &mut World,
) {
    world.insert_resource(CollisionOptimizer::<T>::new());

    schedule
        .add_systems(clear_old_collisions::<T>)
        .add_systems(
            update_collisions_with_rotation::<T>.after(update_rotations_with_angular_velocity),
        )
        .add_systems(update_collisions_with_position::<T>.after(update_positions_with_velocity))
        .add_systems(
            collision_system::<T>
                .after(clear_old_collisions::<T>)
                .after(update_collisions_with_position::<T>)
                .after(update_collisions_with_rotation::<T>)
                .before(post_collision_checkpoint),
        );
}

fn post_collision_checkpoint() {}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Directory to host the webapp from. If ommitted, server is started without.
    #[arg(long)]
    webapp_directory: Option<PathBuf>,

    /// Directory to load gamedata from.
    data_directory: PathBuf,

    /// Display more in-depth logs
    #[clap(long, action)]
    verbose_logs: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let tracing_filters = tracing_subscriber::filter::Targets::new()
        .with_default(Level::TRACE)
        .with_target("bevy_ecs", Level::WARN);

    let tracing = match args.verbose_logs {
        true => FmtSubscriber::builder().with_max_level(Level::TRACE),
        false => FmtSubscriber::builder().with_max_level(Level::INFO),
    }
    .finish()
    .with(tracing_filters);

    tracing::subscriber::set_global_default(tracing).expect("Failed to initialize trace logging");

    let data_directory = match tokio::fs::canonicalize(args.data_directory).await {
        Ok(canon) => canon,
        Err(canon_error) => {
            panic!("Data directory not found! {}", canon_error.to_string());
        }
    };

    debug!(
        "Using data directory: {}",
        data_directory.to_string_lossy()
    );

    let asset_loader =
        match AssetBundleLoader::load_from_directory(data_directory.join("assets"))
            .await
        {
            Ok(ok) => ok,
            Err(()) => {
                panic!("Could not load asset bundles from disk");
            }
        };

    let mut asset_cache = AssetFileCache::new();

    for bundle in asset_loader.get_assets() {
        tracing::debug!("Loading asset bundle {}", bundle.path.to_string_lossy());
        match asset_cache.load_asset_bundle(bundle).await {
            Ok(()) => (),
            Err(()) => {
                panic!("Could not load asset bundle from disk: {}", bundle.path.to_string_lossy());
            },
        }
    }

    match asset_cache.verify_assets() {
        Ok(()) => (),
        Err(()) => {
            panic!("Asset bundles currently loaded failed verification");
        }
    }

    let mut definition_file_cache = DefinitionFileCache::new();

    let definition_loader =
    match AssetBundleLoader::load_from_directory(data_directory.join("definitions"))
        .await
    {
        Ok(ok) => ok,
        Err(()) => {
            panic!("Could not load definition bundles from disk");
        }
    };

    for bundle in definition_loader.get_assets() {
        tracing::debug!("Loading definition bundle {}", bundle.path.to_string_lossy());
        match definition_file_cache.load_definition_bundle(bundle).await {
            Ok(()) => (),
            Err(()) => {
                panic!("Could not load definition bundle from disk: {}", bundle.path.to_string_lossy());
            },
        }
    }

    let app = Router::new();

    let app = match &args.webapp_directory {
        Some(webapp_directory) => {
            app.nest_service("/", ServeDir::new(tokio::fs::canonicalize(webapp_directory).await.unwrap())).layer(CompressionLayer::new())
        }
        None => app,
    };

    let (user_session_sender, user_session_receiver) = unbounded::<UserSession>();

    tokio::spawn(async move {
        let mut world = World::new();
        world.insert_resource(DeltaTResource::new());
        world.insert_resource(ConnectingUsersQueue::new(user_session_receiver));

        let mut schedule = Schedule::default();
        schedule.add_systems(
            (
                increment_time,
                update_rotations_with_angular_velocity,
                update_velocities_with_semi_newtonian_physics,
                update_positions_with_velocity,
            )
                .chain(),
        );

        build_collision_phase::<Displayable>(&mut schedule, &mut world);

        schedule
            .add_systems(post_collision_checkpoint)
            .add_systems(tick_viewport.after(post_collision_checkpoint))
            .add_systems(spawn_a_ship_idk.after(post_collision_checkpoint))
            .add_systems(check_despawn_times.after(post_collision_checkpoint))
            .add_systems(spawn_user_sessions.after(post_collision_checkpoint))
            .add_systems(check_alive_sessions.after(post_collision_checkpoint))
            .add_systems(spawn_player_ship_and_viewports.after(post_collision_checkpoint))
            .add_systems(process_incoming_messages.after(post_collision_checkpoint))
            .add_systems(
                apply_player_control::<SemiNewtonianPhysicsComponent>
                    .after(process_incoming_messages),
            );

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
                trace!(
                    "Ticked in {} milliseconds average with {} entities currently",
                    average_time / STATS_INTERVAL as f32,
                    world.entities().len()
                );
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
        connections: user_session_sender,
    };

    let asset_server_state = AssetServerState {
        assets: Arc::new(asset_cache)
    };

    let app = app
        .route("/ws", get(websocket_handler))
        .with_state(websocket_state)
        .route("/assets/name/:asset_name", get(asset_by_name))
        .with_state(asset_server_state)
        ;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:2718").await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}

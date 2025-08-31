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
mod utility;

use axum::routing::post;
use axum::{Router, routing::get};
use backend::configuration_file_loaders::asset_bundle_loader::AssetBundleLoader;
use backend::resources::delta_t_resource::{DeltaTResource, increment_time};
use backend::spatial_optimizer::collision_optimizer::{CollisionOptimizer, collision_system};
use backend::spatial_optimizer::hash_sized::HashSized;
use backend::world_objects::components::collision_component::clear_old_collisions;
use backend::world_objects::components::semi_newtonian_physics_component::SemiNewtonianPhysicsComponent;
use backend::world_objects::components::timeout_component::{
    TimeoutComponent, check_despawn_times,
};
use backend::world_objects::server_viewport::{Displayable, tick_viewport};
use backend::world_objects::ship::ShipBundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Commands, Res, Resource};
use bevy_ecs::query::With;
use bevy_ecs::schedule::{IntoScheduleConfigs, Schedule};
use bevy_ecs::system::Query;
use bevy_ecs::world::World;
use clap::Parser;
use connectivity::handlers::player_profile_handlers::{
    create_new_ephemeral_player, create_new_username_player,
};
use connectivity::handlers::player_session_handlers::login_player;
use connectivity::player_info::player_profiles::PlayerProfiles;
use connectivity::player_info::player_sessions::PlayerSessions;
use euclid::Angle;
use rand::Rng;
use shared_types::{Coordinates, Speed, Velocity};
use tokio::time;
use tower_http::compression::CompressionLayer;
use tracing::{Level, debug, trace};
use tracing_subscriber::FmtSubscriber;

use connectivity::handlers::websocket_handler::*;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::layer::SubscriberExt;

use tower_http::services::ServeDir;

use crate::backend::configuration_file_loaders::asset_file_cache::AssetFileCache;
use crate::backend::configuration_file_loaders::definition_caches::list_required_assets::ListRequiredAssets;
use crate::backend::configuration_file_loaders::definition_file_cache::DefinitionFileCache;
use crate::backend::resources::delta_t_resource::MINIMUM_TICK_DURATION;
use crate::backend::systems::apply_player_control::apply_player_control;
use crate::backend::systems::player_session_cleanup::player_session_cleanup;
use crate::backend::systems::player_spawn_system::spawn_player_ship_and_viewports;
use crate::backend::systems::submit_command::process_external_commands;
use crate::backend::systems::update_collisions_with_position::update_collisions_with_position;
use crate::backend::systems::update_collisions_with_rotation::update_collisions_with_rotation;
use crate::backend::systems::update_positions_with_velocity::update_positions_with_velocity;
use crate::backend::systems::update_rotations_with_angular_velocity::update_rotations_with_angular_velocity;
use crate::backend::systems::update_velocities_with_semi_newtonian_physics::update_velocities_with_semi_newtonian_physics;
use crate::backend::world_objects::components::random_ship_spawn_placeholder::RandomShipSpawnPlaceholderComponent;
use crate::backend::world_objects::planetoid::PlanetoidBundle;
use crate::connectivity::asset_index::{AssetIndex, AssetIndexState, get_asset_index};
use crate::connectivity::asset_server::{AssetServerState, asset_by_name};
use crate::connectivity::handlers::chat_handlers::{send_message, subscribe_message};
use crate::connectivity::services::chat_service::ChatService;
use crate::connectivity::services::ecs_communication_service::EcsCommunicationService;

fn plus_or_minus_random(radius: f64) -> f64 {
    let value = rand::rng().random::<f64>();
    let range = radius * 2.0;
    (range * value) - radius
}

#[derive(Resource)]
struct AssetIndexResource {
    asset_index: Arc<AssetIndex>,
}

async fn spawn_a_ship_idk_task(commands: EcsCommunicationService) {
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _ = commands
            .run_command(|commands| -> Result<(), ()> {
                commands.spawn(RandomShipSpawnPlaceholderComponent {});
                Ok(())
            })
            .await
            .unwrap()
            .unwrap();
    }
}

fn spawn_a_ship_idk(
    placeholders: Query<Entity, With<RandomShipSpawnPlaceholderComponent>>,
    asset_index: Res<AssetIndexResource>,
    time: Res<DeltaTResource>,
    mut commands: Commands,
) {
    for spawn in placeholders.iter() {
        commands.spawn((
            ShipBundle::new(
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
                &asset_index.asset_index,
            )
            .unwrap(),
            SemiNewtonianPhysicsComponent::new(Speed::new(50.0)),
            TimeoutComponent {
                spawn_time: time.total_time,
                lifetime: Duration::from_secs(10),
            },
        ));

        commands.entity(spawn).despawn();
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

fn pre_collision_checkpoint() {}
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

    /// Verify all required assets are loaded for the definitions
    #[clap(long, action)]
    verify_assets: bool,
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

    debug!("Using data directory: {}", data_directory.to_string_lossy());

    let asset_loader =
        match AssetBundleLoader::load_from_directory(data_directory.join("assets")).await {
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
                panic!(
                    "Could not load asset bundle from disk: {}",
                    bundle.path.to_string_lossy()
                );
            }
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
        match AssetBundleLoader::load_from_directory(data_directory.join("definitions")).await {
            Ok(ok) => ok,
            Err(()) => {
                tracing::error!("Could not load definition bundles from disk");
                panic!("Could not load definition bundles from disk");
            }
        };

    for bundle in definition_loader.get_assets() {
        tracing::debug!(
            "Loading definition bundle {}",
            bundle.path.to_string_lossy()
        );
        match definition_file_cache.load_definition_bundle(bundle).await {
            Ok(()) => (),
            Err(()) => {
                tracing::error!(
                    "Could not load definition bundle from disk: {}",
                    bundle.path.to_string_lossy()
                );
                panic!(
                    "Could not load definition bundle from disk: {}",
                    bundle.path.to_string_lossy()
                );
            }
        }
    }

    if args.verify_assets {
        let mut loading_error = false;
        for required_asset in definition_file_cache.get_required_asset_list() {
            match asset_cache.get_asset_definition_by_name(required_asset.0) {
                Some(has_asset) => {
                    if has_asset.asset_type.get_asset_type_from_resource() == required_asset.1 {
                        // All good
                    } else {
                        tracing::error!(
                            "Mismatch between the type of loaded asset {} with {:?} being loaded and {:?} being required",
                            required_asset.0,
                            has_asset,
                            required_asset.1
                        );
                        loading_error = true;
                    }
                }
                None => {
                    tracing::error!(
                        "Loaded asset bundles do not contain required asset {} of type {:?}",
                        required_asset.0,
                        required_asset.1
                    );
                    loading_error = true;
                }
            }
        }

        if loading_error {
            tracing::error!("Not all required assets are loaded or correct!");
            panic!("Not all required assets are loaded or correct!");
        }
    }

    let asset_index = Arc::new(AssetIndex::new(
        definition_file_cache
            .get_required_asset_list()
            .iter()
            .map(|asset| asset.0)
            .cloned(),
    ));

    let app = Router::new();

    let app = match &args.webapp_directory {
        Some(webapp_directory) => app
            .fallback_service(ServeDir::new(
                tokio::fs::canonicalize(webapp_directory).await.unwrap(),
            ))
            .layer(CompressionLayer::new()),
        None => app,
    };

    let resource_asset_index = asset_index.clone();

    let (web_ecs_command_service, ecs_ecs_command_resource) = EcsCommunicationService::create();

    tokio::task::spawn_blocking(move || {
        let mut world = World::new();

        world.spawn_batch(
            definition_file_cache
                .get_planetoids()
                .iter()
                .map(|planetoid| PlanetoidBundle::new(planetoid, &resource_asset_index).unwrap()),
        );
        world.insert_resource(DeltaTResource::new());
        world.insert_resource(AssetIndexResource {
            asset_index: resource_asset_index,
        });
        world.insert_resource(ecs_ecs_command_resource);

        let mut schedule = Schedule::default();

        schedule.add_systems(
            (
                player_session_cleanup,
                process_external_commands,
                pre_collision_checkpoint,
            )
                .chain(),
        );

        schedule.add_systems(
            (
                increment_time,
                update_rotations_with_angular_velocity,
                update_velocities_with_semi_newtonian_physics,
                update_positions_with_velocity,
                post_collision_checkpoint,
            )
                .chain()
                .after(pre_collision_checkpoint),
        );

        build_collision_phase::<Displayable>(&mut schedule, &mut world);

        schedule
            .add_systems(
                (
                    tick_viewport,
                    spawn_a_ship_idk,
                    check_despawn_times,
                    spawn_player_ship_and_viewports,
                )
                    .after(post_collision_checkpoint),
            )
            .add_systems(
                apply_player_control::<SemiNewtonianPhysicsComponent>
                    .after(post_collision_checkpoint),
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
            spin_sleep::sleep(minimum_time);
        }
    });

    let asset_server_state = AssetServerState {
        assets: Arc::new(asset_cache),
    };

    let asset_index_state = AssetIndexState {
        assets: asset_index,
    };

    let player_profile_state = PlayerProfiles::new();
    let player_session_state = PlayerSessions::default();
    let chat_service = ChatService::default();

    let websocket_state = HandlerState {
        sessions: player_session_state.clone(),
    };

    tokio::spawn(spawn_a_ship_idk_task(web_ecs_command_service.clone()));

    let app = app
        .route("/ws", get(websocket_handler))
        .with_state(websocket_state)
        .route("/assets/name/{asset_name}", get(asset_by_name))
        .with_state(asset_server_state)
        .route("/assets/index", get(get_asset_index))
        .with_state(asset_index_state)
        .route(
            "/players/newephemeralplayer",
            post(create_new_ephemeral_player),
        )
        .route("/players/newplayer", post(create_new_username_player))
        .with_state(player_profile_state.clone())
        .route("/players/login", post(login_player))
        .with_state((
            player_profile_state,
            player_session_state.clone(),
            web_ecs_command_service.clone(),
        ))
        .route("/players/messaging/send-message", post(send_message))
        .with_state((chat_service.clone(), player_session_state.clone()))
        .route(
            "/players/messaging/subscribe-message",
            get(subscribe_message),
        )
        .with_state((chat_service.clone(), player_session_state.clone()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:2718").await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

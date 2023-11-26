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
use clap::Parser;
use connectivity::connected_users::ConnectedUsers;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use std::net::SocketAddr;
use std::sync::*;
use connectivity::websocket_handler::*;

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
    verbose_logs: bool
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let tracing = match args.verbose_logs {
        true => {
            FmtSubscriber::builder().with_max_level(Level::TRACE)
        },
        false => {
            FmtSubscriber::builder().with_max_level(Level::INFO)
        }
    };

    tracing::subscriber::set_global_default(tracing.finish()).expect("Failed to initialize trace logging");

    let app = Router::new();

    let app = match &args.webapp_directory {
        Some(webapp_directory) => app.nest_service("/", ServeDir::new(webapp_directory)),
        None => app,
    };

    let user_connections = Arc::new(ConnectedUsers::new());

    tokio::spawn(ConnectedUsers::garbage_collector(
        user_connections.clone(),
    ));

    let app = app
        .route("/ws", get(websocket_handler))
        .with_state(user_connections.clone());

    axum::Server::bind(&"0.0.0.0:2718".parse().unwrap())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
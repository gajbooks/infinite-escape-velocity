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

use actix_files::Files;
use actix_web::{App, HttpServer};
use clap::Parser;
use connectivity::client_server_message::*;
use connectivity::server_client_message::*;

use shared_types::*;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::sync::*;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Directory to host the webapp from. If ommitted, server is started in dedicated mode.
    #[arg(long)]
    webapp_directory: Option<String>,

    /// Directory to load gamedata from.
    data_directory: String
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    HttpServer::new(move || {
        let app = App::new();
        let app = match &args.webapp_directory {
            Some(webapp_directory) => {
                app.service(Files::new("/", webapp_directory))
            },
            None => {
                app
            }
        };

        app
        })
        .bind("0.0.0.0:6969")?
        .run()
        .await
}

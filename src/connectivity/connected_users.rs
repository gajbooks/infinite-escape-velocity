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

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use bevy_ecs::system::{Commands, Res, Resource};
use tokio::time;

use crate::{
    backend::world_objects::{
        object_properties::collision_component::CollidableComponent,
        server_viewport::{ServerViewport, ViewportBundle},
    },
    connectivity::user_session::UserSession,
};

pub fn create_user_viewports(connected_users: Res<ConnectedUsersResource>, mut commands: Commands) {
    let connected_users = connected_users
        .connected_users
        .connection_list
        .lock()
        .unwrap();
    for user in connected_users.iter() {
        let mut viewports = user.viewports_to_spawn.lock().unwrap();
        for new_viewports in viewports.iter() {
            commands.spawn(ViewportBundle {
                viewport: ServerViewport::new(user.cancel.clone(), user.to_remote.clone()),
                collidable: CollidableComponent::new(new_viewports.clone()),
            });
        }

        viewports.clear();
    }
}

pub struct ConnectedUsers {
    pub connection_list: Mutex<Vec<Arc<UserSession>>>,
}

#[derive(Resource)]
pub struct ConnectedUsersResource {
    pub connected_users: Arc<ConnectedUsers>,
}

impl ConnectedUsers {
    pub fn new() -> ConnectedUsers {
        ConnectedUsers {
            connection_list: Mutex::new(Vec::new()),
        }
    }

    pub fn add_user(&self, new_user: Arc<UserSession>) {
        let mut connected_users = self.connection_list.lock().unwrap();
        tracing::info!("New user connected from {}", new_user.remote_address);
        connected_users.push(new_user);
    }

    pub async fn garbage_collector(connected_users: Arc<ConnectedUsers>) {
        let mut interval = time::interval(Duration::from_millis(500));

        loop {
            interval.tick().await;
            {
                let mut list = connected_users.connection_list.lock().unwrap();
                list.retain(|x| !x.is_dead());
            }
        }
    }
}

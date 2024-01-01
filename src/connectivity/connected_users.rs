use std::{sync::{Mutex, Arc}, time::Duration};

use bevy_ecs::system::{Resource, Commands, Res};
use tokio::time;

use crate::{connectivity::user_session::UserSession, backend::world_objects::{server_viewport::{ViewportBundle, ServerViewport}, object_properties::collision_component::{CollidableComponent}}};

pub fn create_user_viewports(connected_users: Res<ConnectedUsersResource>, mut commands: Commands) {
    let connected_users = connected_users.connected_users.connection_list.lock().unwrap();
    for user in connected_users.iter() {
        let mut viewports = user.viewports_to_spawn.lock().unwrap();
        for new_viewports in viewports.iter() {
            commands.spawn(ViewportBundle{
                viewport: ServerViewport::new(user.cancel.clone(), user.to_remote.clone()),
                collidable: CollidableComponent::new(new_viewports.clone())});
        }

        viewports.clear();
    }
}

pub struct ConnectedUsers {
    pub connection_list: Mutex<Vec<Arc<UserSession>>>,
}

#[derive(Resource)]
pub struct ConnectedUsersResource {
    pub connected_users: Arc<ConnectedUsers>
}

impl ConnectedUsers {
    pub fn new() -> ConnectedUsers {
        ConnectedUsers { connection_list: Mutex::new(Vec::new()) }
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
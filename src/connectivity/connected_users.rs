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

use std::sync::atomic::Ordering;

use bevy_ecs::{
    entity::Entity,
    system::{Commands, ParallelCommands, Query, ResMut, Resource},
};
use futures::channel::mpsc::UnboundedReceiver;

use crate::connectivity::user_session::UserSession;

pub fn spawn_user_sessions(
    mut connecting_users: ResMut<ConnectingUsersQueue>,
    mut commands: Commands,
) {
    while let Ok(new_user) = connecting_users.new_users.try_next() {
        match new_user {
            Some(new_user) => {
                tracing::info!("New user connected from {}", new_user.remote_address);
                commands.spawn(new_user);
            }
            None => {
                // Disconnected
                tracing::error!(
                    "User session spawning resource became detached from connection management"
                );
                panic!("Cannot spawn new user sessions with connection management detached");
            }
        }
    }
}

pub fn check_alive_sessions(sessions: Query<(Entity, &UserSession)>, commands: ParallelCommands) {
    sessions.par_iter().for_each(|(entity, session)| {
        if session.cancel.load(Ordering::Relaxed) {
            commands.command_scope(|mut commands| {
                commands.entity(entity).despawn();
            });
        }
    })
}

#[derive(Resource)]
pub struct ConnectingUsersQueue {
    pub new_users: UnboundedReceiver<UserSession>,
}

impl ConnectingUsersQueue {
    pub fn new(new_users: UnboundedReceiver<UserSession>) -> ConnectingUsersQueue {
        ConnectingUsersQueue { new_users }
    }
}

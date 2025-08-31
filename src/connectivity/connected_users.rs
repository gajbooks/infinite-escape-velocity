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

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, ParallelCommands, Query, ResMut, Resource},
};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::connectivity::user_session::UserSession;

pub fn spawn_user_sessions(
    mut connecting_users: ResMut<ConnectingUsersQueue>,
    mut commands: Commands,
) {
    loop {
        let attempt_message = connecting_users.new_users.try_recv();
        match attempt_message {
            Ok(new_user) => {
                tracing::info!(
                    "New user connected from {}",
                    new_user.websocket_connection.remote_address
                );
                commands.spawn(new_user);
            }
            Err(e) => {
                match e {
                    tokio::sync::mpsc::error::TryRecvError::Empty => {
                        // No messages we can immediately receive, exit and retry again next ECS run
                        break;
                    }
                    tokio::sync::mpsc::error::TryRecvError::Disconnected => {
                        // Disconnected
                        tracing::error!(
                            "User session spawning resource became detached from connection management"
                        );

                        panic!(
                            "Cannot spawn new user sessions with connection management detached"
                        );
                    }
                }
            }
        }
    }
}

pub fn check_alive_sessions(sessions: Query<(Entity, &UserSession)>, commands: ParallelCommands) {
    sessions.par_iter().for_each(|(entity, session)| {
        if session.websocket_connection.cancel.is_canceled() {
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

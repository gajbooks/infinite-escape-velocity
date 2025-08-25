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
    resource::Resource,
    system::{Commands, Res},
};
use tokio::sync::{Mutex, mpsc};
use tracing::warn;

use crate::utility::async_handle::AsyncSupplier;

pub trait EcsExternalCommand: (FnOnce(&mut Commands) -> CommandResult) + Send + 'static {}

impl<T> EcsExternalCommand for T where T: FnOnce(&mut Commands) -> CommandResult + Send + 'static {}

pub type CommandResult = Result<(), ()>;
pub type CommandFnBox = Box<dyn EcsExternalCommand>;

#[derive(Resource)]
pub struct EcsCommandQueue {
    receiver: Mutex<mpsc::Receiver<AsyncSupplier<CommandFnBox, CommandResult>>>,
}

impl EcsCommandQueue {
    pub fn new(
        receiver: mpsc::Receiver<AsyncSupplier<CommandFnBox, CommandResult>>,
    ) -> EcsCommandQueue {
        EcsCommandQueue {
            receiver: Mutex::new(receiver),
        }
    }
}

pub fn process_external_commands(queue: Res<EcsCommandQueue>, mut commands: Commands) {
    let mut locked_queue = queue.receiver.blocking_lock();
    while let Ok(mut command) = locked_queue.try_recv() {
        let result = command
            .get_arguments()
            .map(|x| x(&mut commands))
            .ok_or(())
            .flatten();

        let submit_result = command.submit_results(result);

        if let Err(_) = submit_result {
            warn!("Command failed to submit result! Duplicate command submission possible.");
        }
    }
}

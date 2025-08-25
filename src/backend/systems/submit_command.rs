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

use std::any::Any;

use bevy_ecs::{
    resource::Resource,
    system::{Commands, Res},
};
use tokio::sync::{Mutex, mpsc};

use crate::utility::async_handle::{AsyncSupplier, AsyncSupplierCallback};

pub trait EcsExternalCommand<U: EcsExternalCommandResult>: FnOnce(&mut Commands) -> U + Send + 'static {}
impl<T, U> EcsExternalCommand<U> for T where T: FnOnce(&mut Commands) -> U + Send + 'static, U: EcsExternalCommandResult {}

pub trait EcsExternalCommandResult: Send + 'static {}
impl<T> EcsExternalCommandResult for T where T: Send + 'static {}

pub trait EcsExternalCommandProxy: FnOnce(Box<dyn AsyncSupplierCallback<CommandResult>>, &mut Commands) -> () + Send + 'static {}
impl<T> EcsExternalCommandProxy for T where T: FnOnce(Box<dyn AsyncSupplierCallback<CommandResult>>, &mut Commands) -> () + Send + 'static {}

pub type CommandResult = Box<dyn Any + Send>;
pub type CommandFnBox = Box<dyn EcsExternalCommandProxy>;

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
        if let Some(arguments_exist) = command.get_arguments() {
            arguments_exist(Box::new(command), &mut commands);
        }


    }
}

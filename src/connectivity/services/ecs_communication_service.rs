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

use tokio::sync::mpsc;

use crate::{
    backend::systems::submit_command::{
        CommandFnBox, CommandResult, EcsCommandQueue, EcsExternalCommand,
    },
    utility::async_handle::{AsyncSupplier, async_handle},
};

const COMMAND_BACKPRESSURE_LIMIT: usize = 1000;

#[derive(Clone)]
pub struct EcsCommunicationService {
    sender: mpsc::Sender<AsyncSupplier<CommandFnBox, CommandResult>>,
}

impl EcsCommunicationService {
    pub async fn run_command<T: EcsExternalCommand>(&self, command: T) -> CommandResult {
        let (handle, supplier) = async_handle::<CommandFnBox, CommandResult>(Box::new(command));
        self.sender.send(supplier).await.unwrap();
        handle.receive().await.unwrap()
    }

    pub fn create() -> (EcsCommunicationService, EcsCommandQueue) {
        let (sender, receiver) =
            mpsc::channel::<AsyncSupplier<CommandFnBox, CommandResult>>(COMMAND_BACKPRESSURE_LIMIT);
        (
            EcsCommunicationService { sender },
            EcsCommandQueue::new(receiver),
        )
    }
}

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

use bevy_ecs::system::Commands;
use tokio::sync::{mpsc, oneshot::error::RecvError};
use tracing::{error, warn};

use crate::{
    backend::systems::submit_command::{
        CommandFnBox, CommandResult, EcsCommandQueue, EcsExternalCommand, EcsExternalCommandResult,
    },
    utility::async_handle::{async_handle, AsyncSupplier, AsyncSupplierCallback},
};

const COMMAND_BACKPRESSURE_LIMIT: usize = 1000;

#[derive(Clone)]
pub struct EcsCommunicationService {
    sender: mpsc::Sender<AsyncSupplier<CommandFnBox, CommandResult>>,
}

impl EcsCommunicationService {
    pub async fn run_command<U: EcsExternalCommandResult, T: EcsExternalCommand<U>>(&self, command: T) -> Result<U, RecvError> {
        let wrapper = |mut callback: Box<dyn AsyncSupplierCallback<CommandResult>>,
                       commands: &mut Commands| {
            let result = command(commands);
            let submit_result = callback.submit_results(Box::new(result));

            if let Err(_) = submit_result {
                warn!("Command failed to submit result! Called submit on an already taken value.");
            }
        };

        let (handle, supplier) = async_handle::<CommandFnBox, CommandResult>(Box::new(wrapper));
        self.sender.send(supplier).await.unwrap();

        match handle.receive().await {
            Ok(return_value) => {
                // This unwrap should be statically guaranteed because we are not going to return a different type U than we took as an argument
                Ok(*return_value.downcast().unwrap())
            },
            Err(e) => {
                error!("ECS Communication Service had a command dropped inside the ECS without executing!");
                Err(e)
            },
        }
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

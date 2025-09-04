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

use tokio::sync::oneshot::{
    self,
    error::{RecvError, TryRecvError},
};

pub fn async_handle<U, T>(arguments: U) -> (AsyncHandle<T>, AsyncSupplier<U, T>) {
    let (tx, rx) = oneshot::channel();
    (
        AsyncHandle { receiver: rx },
        AsyncSupplier {
            arguments: Some(arguments),
            sender: Some(tx),
        },
    )
}

pub struct AsyncHandle<T> {
    receiver: oneshot::Receiver<T>,
}

impl<T> AsyncHandle<T> {
    pub async fn blocking_receive(self) -> Result<T, RecvError> {
        self.receiver.blocking_recv()
    }

    pub async fn receive(self) -> Result<T, RecvError> {
        self.receiver.await
    }

    pub async fn poll_receive(mut self) -> Result<T, TryRecvError> {
        self.receiver.try_recv()
    }
}

pub trait AsyncSupplierCallback<T> {
    fn submit_results(&mut self, value: T) -> Result<(), T>;
}

pub struct AsyncSupplier<U, T> {
    arguments: Option<U>,
    sender: Option<oneshot::Sender<T>>,
}

impl<U, T> AsyncSupplier<U, T> {
    pub fn get_arguments(&mut self) -> Option<U> {
        self.arguments.take()
    }

    pub fn submit_results(&mut self, value: T) -> Result<(), T> {
        match self.sender.take() {
            Some(x) => x.send(value),
            None => Err(value),
        }
    }
}

impl<U, T> AsyncSupplierCallback<T> for AsyncSupplier<U, T> {
    fn submit_results(&mut self, value: T) -> Result<(), T> {
        self.submit_results(value)
    }
}

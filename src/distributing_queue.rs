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

use dashmap::DashMap;
use dashmap::mapref::entry::*;
use fxhash::FxBuildHasher;
use super::identifiable_object::*;
use crossbeam_channel::*;

pub struct DistributingQueue<T: Clone> {
    channel_sender: crossbeam_channel::Sender<SendTo<T>>,
    channel_receiver: crossbeam_channel::Receiver<SendTo<T>>,
    registered_receivers: DashMap<IdType, crossbeam_channel::Sender<SentFrom<T>>, FxBuildHasher>
}

impl <T: Clone> DistributingQueue<T> {
    pub fn new() -> DistributingQueue<T> {
        let (s, r) = unbounded();
        return DistributingQueue {channel_sender: s, channel_receiver: r, registered_receivers: DashMap::with_hasher(FxBuildHasher::default())};
    }

    pub fn register_receiver(&self, id: IdType, destination_channel: crossbeam_channel::Sender<SentFrom<T>>) {
        self.registered_receivers.insert(id, destination_channel);
    }

    pub fn get_sender(&self) -> crossbeam_channel::Sender<SendTo<T>> {
        return self.channel_sender.clone();
    }

    pub fn process_queue(&self) {
        match self.channel_receiver.try_recv() {
            Ok(val) => {
                match self.registered_receivers.entry(val.destination_id) {
                    Entry::Occupied(registered) => {
                        match registered.get().send(val.from) {
                            Ok(_v) => (),
                            Err(_e) => {
                                // Remove queues that error (disconnected)
                                registered.remove();
                            }
                        }
                    },
                    Entry::Vacant(_v) => {
                        // Is not registered
                    }
                }
            },
            Err(_e) => {
                // Queue is empty, cannot be disconnected as it owns its own queue
            }
        }

        super::shrink_storage!(self.registered_receivers);
    }
}
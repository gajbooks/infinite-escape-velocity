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

use super::hash_coordinates::*;
use super::identifiable_object::*;
use super::shape::*;
use crossbeam_channel::unbounded;
use dashmap::DashMap;
use fxhash::FxBuildHasher;
use std::sync::Arc;

pub enum SpatialMessage {
    Add { new: SentFrom<Arc<Shape>> },
    Delete { old: IdType },
}

pub struct SpatialHashmap {
    map: DashMap<HashCoordinates, Vec<SentFrom<Arc<Shape>>>, FxBuildHasher>,
    current_registered_shapes: DashMap<IdType, Arc<Shape>, FxBuildHasher>,
    channel_sender: crossbeam_channel::Sender<SpatialMessage>,
    channel_receiver: crossbeam_channel::Receiver<SpatialMessage>,
    global_collision_distributor: crossbeam_channel::Sender<SendTo<Arc<Shape>>>,
}

impl SpatialHashmap {
    pub fn new(
        collision_distributor: crossbeam_channel::Sender<SendTo<Arc<Shape>>>,
    ) -> SpatialHashmap {
        let (s, r) = unbounded();
        return SpatialHashmap {
            map: DashMap::with_hasher(FxBuildHasher::default()),
            current_registered_shapes: DashMap::with_hasher(FxBuildHasher::default()),
            channel_sender: s,
            channel_receiver: r,
            global_collision_distributor: collision_distributor,
        };
    }

    pub fn get_channel(&self) -> crossbeam_channel::Sender<SpatialMessage> {
        return self.channel_sender.clone();
    }

    fn add(&self, add: &SentFrom<Arc<Shape>>) -> () {
        let mut dedup = Vec::<SentFrom<Arc<Shape>>>::new();

        match self.current_registered_shapes.entry(add.origin_id) {
            dashmap::mapref::entry::Entry::Occupied(has) => {
                let removed = has.replace_entry(add.data.clone());
                self.remove_old_entries_from_map(&removed.0, &removed.1);
            }
            dashmap::mapref::entry::Entry::Vacant(not) => {
                // Entry not already present
                not.insert(add.data.clone());
            }
        }

        for coordinates in add.data.aabb_iter() {
            let mut entry = self.map.entry(coordinates).or_default();
            for i in entry.value() {
                dedup.push(i.clone());
            }

            entry.push(add.clone());
        }

        dedup.sort_by(|a, b| a.origin_id.cmp(&b.origin_id));
        dedup.dedup_by(|a, b| a.origin_id.eq(&b.origin_id));

        for collision in dedup {
            if collision.data.collides(&add.data) {
                // Send old entity collisions to new entity
                match self.global_collision_distributor.send(SendTo {
                    destination_id: add.origin_id,
                    from: collision.clone(),
                }) {
                    Ok(_) => {
                        //Sent successfully
                    }
                    Err(_e) => {
                        // New entity disconnected channel before deletion processed, queue for deletion now
                        match self
                            .channel_sender
                            .send(SpatialMessage::Delete { old: add.origin_id })
                        {
                            Ok(_) => {
                                // Sent successfully
                            }
                            Err(_e) => {
                                // Cannot disconnect because this object owns both ends
                            }
                        }
                    }
                }

                // Send new entity collisions to old entity
                match self.global_collision_distributor.send(SendTo {
                    destination_id: collision.origin_id,
                    from: add.clone(),
                }) {
                    Ok(_) => {
                        //Sent successfully
                    }
                    Err(_e) => {
                        // Entity disconnected channel before deletion processed, queue for deletion now
                        match self.channel_sender.send(SpatialMessage::Delete {
                            old: collision.origin_id,
                        }) {
                            Ok(_) => {
                                // Sent successfully
                            }
                            Err(_e) => {
                                // Cannot disconnect because this object owns both ends
                            }
                        }
                    }
                }
            }
        }
    }

    fn remove_old_entries_from_map(
        &self, id: &IdType, old: &Arc<Shape>
    ) {
        for coordinates in old.aabb_iter() {
            match self.map.get_mut(&coordinates) {
                Some(mut exists) => {
                    exists.retain(|x| x.origin_id != *id);
                    super::shrink_storage!(exists);
                }
                None => {
                    // ???
                    println!("Tried to remove entry from hashmap that was never entered?");
                }
            }
            self.map.remove_if(&coordinates, |_, list| list.is_empty());
        }

        super::shrink_storage!(self.map);
    }

    fn remove(&self, old: IdType) -> () {
        match self.current_registered_shapes.entry(old) {
            dashmap::mapref::entry::Entry::Occupied(has) => {
                self.remove_old_entries_from_map(has.key(), has.get());
                has.remove();
            },
            dashmap::mapref::entry::Entry::Vacant(_not) => {
                // Tried to remove entry that is not present, does not matter
                println!("Tried to remove ID that was never entered?");
            }
        }

        super::shrink_storage!(self.current_registered_shapes);
    }

    pub fn process_entry(&self) {
        match self.channel_receiver.try_recv() {
            Ok(val) => match val {
                SpatialMessage::Add { new } => {
                    self.add(&new);
                }
                SpatialMessage::Delete { old } => {
                    self.remove(old);
                }
            },
            Err(_e) => {
                // Queue is empty, cannot be disconnected as it owns its own queue
            }
        }
    }
}

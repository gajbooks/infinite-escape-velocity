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

use crate::backend::world_objects::components::collision_component::{
    CollisionEvaluatorComponent, CollisionSourceComponent,
};

use super::{hash_cell_size::HashCellSize, hash_coordinates::*};
use bevy_ecs::prelude::*;
use rayon::prelude::*;

enum SenderReceiver<'a, T: ?Sized + HashCellSize> {
    Sender(&'a CollisionSourceComponent<T>),
    Receiver(&'a CollisionEvaluatorComponent<T>),
}

struct ObjectWithinCell<'a, T: ?Sized + HashCellSize> {
    pub cell: HashCoordinates,
    pub entity: Entity,
    pub sender_receiver: SenderReceiver<'a, T>,
}

pub fn collision_system<T: ?Sized + HashCellSize>(
    mut optimizer: ResMut<CollisionOptimizer<T>>,
    receivers: Query<(Entity, &CollisionEvaluatorComponent<T>)>,
    senders: Query<(Entity, &CollisionSourceComponent<T>)>,
) {
    let mut list = optimizer.cache.take().unwrap();
    list.extend(receivers.iter().flat_map(|(entity, collision_receiver)| {
        collision_receiver
            .shape
            .aabb_iter(T::HASH_CELL_SIZE)
            .map(move |coordinates| ObjectWithinCell {
                cell: coordinates,
                entity: entity.clone(),
                sender_receiver: SenderReceiver::Receiver(collision_receiver),
            })
    }));

    list.extend(senders.iter().flat_map(|(entity, collision_sender)| {
        collision_sender
            .shape
            .aabb_iter(T::HASH_CELL_SIZE)
            .map(move |coordinates| ObjectWithinCell {
                cell: coordinates,
                entity: entity.clone(),
                sender_receiver: SenderReceiver::Sender(collision_sender),
            })
    }));

    list.par_sort_unstable_by(|x, y| x.cell.cmp(&y.cell));

    list.par_iter().enumerate().for_each(|range| {
        let outer_object = &range.1;

        let mut inner_index = range.0 + 1;

        if inner_index >= list.len() {
            return;
        }

        while inner_index < list.len() && outer_object.cell == list[inner_index].cell {
            let inner_object = &list[inner_index];
            match outer_object.sender_receiver {
                SenderReceiver::Sender(sender) => match inner_object.sender_receiver {
                    SenderReceiver::Sender(_) => (),
                    SenderReceiver::Receiver(receiver) => {
                        if sender.shape.collides(&receiver.shape)
                            && (inner_object.entity != outer_object.entity)
                        {
                            receiver.add_to_collision_list(outer_object.entity);
                        }
                    }
                },
                SenderReceiver::Receiver(receiver) => match inner_object.sender_receiver {
                    SenderReceiver::Sender(sender) => {
                        if sender.shape.collides(&receiver.shape)
                            && (inner_object.entity != outer_object.entity)
                        {
                            receiver.add_to_collision_list(inner_object.entity);
                        }
                    }
                    SenderReceiver::Receiver(_) => (),
                },
            };
            inner_index += 1;
        }
    });

    list.clear();
    optimizer.cache = list.into_iter().map(|_| unreachable!()).collect();
}

#[derive(Resource)]
pub struct CollisionOptimizer<T: ?Sized + HashCellSize> {
    cache: Option<Vec<ObjectWithinCell<'static, T>>>,
}

impl<T: ?Sized + HashCellSize> CollisionOptimizer<T> {
    pub fn new() -> Self {
        Self {
            cache: Some(Vec::new()),
        }
    }
}

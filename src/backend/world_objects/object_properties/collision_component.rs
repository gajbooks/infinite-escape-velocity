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

use std::marker::PhantomData;

use bevy_ecs::prelude::*;
use dashmap::DashSet;

use crate::backend::{shape::Shape, shrink_storage::*};

#[derive(Component)]
pub struct CollisionMarker<T: Send + Sync + Sized + 'static> {
    _phantom: PhantomData<T>,
    pub shape: Shape,
}

impl<T: Send + Sync + Sized + 'static> CollisionMarker<T> {
    pub fn new(shape: Shape) -> Self {
        Self {
            _phantom: PhantomData,
            shape: shape,
        }
    }
}

#[derive(Component)]
pub struct CollidableComponent<T: Send + Sync + Sized + 'static> {
    _phantom: PhantomData<T>,
    pub list: DashSet<Entity>,
    pub shape: Shape,
}

pub fn clear_old_collisions<T: Send + Sync + Sized>(collidables: Query<&CollidableComponent<T>>) {
    collidables.par_iter().for_each(|x| x.clear());
}

impl<T: Send + Sync + Sized + 'static> CollidableComponent<T> {
    pub fn clear(&self) {
        self.list.shrink_storage();
        self.list.clear();
    }

    pub fn add_to_collision_list(&self, collided_object: Entity) {
        self.list.insert(collided_object);
    }

    pub fn new(shape: Shape) -> CollidableComponent<T> {
        CollidableComponent {
            _phantom: PhantomData,
            list: DashSet::new(),
            shape: shape,
        }
    }
}

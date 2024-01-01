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

use std::ops::DerefMut;

use dashmap::{DashMap, DashSet};

pub trait ImmutableShrinkable {
    fn shrink_storage(&self);
}

impl<K: std::cmp::Eq + std::hash::Hash, S: std::hash::BuildHasher + Clone> ImmutableShrinkable
    for DashSet<K, S>
{
    fn shrink_storage(&self) {
        self.shrink_to_fit();
    }
}

impl<K: std::cmp::Eq + std::hash::Hash, S> ImmutableShrinkable for DashMap<K, S> {
    fn shrink_storage(&self) {
        self.shrink_to_fit();
    }
}

pub trait MutableShrinkable {
    fn shrink_storage(&mut self);
}

impl<T: DerefMut<Target = Vec<K>>, K> MutableShrinkable for T {
    fn shrink_storage(&mut self) {
        let shrink_size = ::std::cmp::max(::std::cmp::max(self.len() * 2, self.capacity() / 4), 10);
        self.shrink_to(shrink_size);
    }
}

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

use std::sync::{atomic::{self, AtomicBool}, Arc};

#[derive(Clone)]
pub struct CancelFlag(Arc<AtomicBool>);

impl Default for CancelFlag {
    fn default() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }
}

impl CancelFlag {
    pub fn cancel(&self) -> bool {
        self.0.swap(true, atomic::Ordering::Relaxed)
    }

    pub fn is_canceled(&self) -> bool {
        self.0.load(atomic::Ordering::Relaxed)
    }
}

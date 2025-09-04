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

use std::time::{self, Duration};

use bevy_ecs::prelude::Resource;

const MICROSECONDS_PER_SECOND: u64 = 1_000_000;
const FRACTIONAL_MAX_TICK_TIME: u64 = 20;
const FRACTIONAL_MIN_TICK_TIME: u64 = 60;
const MAXIMUM_TICK_MICROSECONDS: u64 = MICROSECONDS_PER_SECOND / FRACTIONAL_MAX_TICK_TIME;
const MINIMUM_TICK_MICROSECONDS: u64 = MICROSECONDS_PER_SECOND / FRACTIONAL_MIN_TICK_TIME;
pub const MAXIMUM_TICK_DURATION: Duration = time::Duration::from_micros(MAXIMUM_TICK_MICROSECONDS);
pub const MINIMUM_TICK_DURATION: Duration = time::Duration::from_micros(MINIMUM_TICK_MICROSECONDS);

#[derive(Resource)]
pub struct DeltaTResource {
    total_time: Duration,
    last_tick: Duration,
}

impl DeltaTResource {
    pub fn new() -> Self {
        Self {
            total_time: MINIMUM_TICK_DURATION,
            last_tick: MINIMUM_TICK_DURATION,
        }
    }

    pub fn get_total_time(&self) -> Duration {
        self.total_time
    }

    pub fn get_last_tick_duration(&self) -> Duration {
        self.last_tick
    }

    pub fn increment_time(&mut self, reported_real_world_time: Duration) {
        let corrected_duration =
            reported_real_world_time.clamp(MINIMUM_TICK_DURATION, MAXIMUM_TICK_DURATION);

        self.total_time = self.total_time + corrected_duration;
        self.last_tick = corrected_duration;
    }
}

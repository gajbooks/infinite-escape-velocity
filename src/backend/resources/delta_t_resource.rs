use std::time::{self, Duration};

use bevy_ecs::system::{ResMut, Resource};

const MICROSECONDS_PER_SECOND: u64 = 1_000_000;
const FRACTIONAL_MAX_TICK_TIME: u64 = 20;
const FRACTIONAL_MIN_TICK_TIME: u64 = 60;
const MAXIMUM_TICK_MICROSECONDS: u64 = MICROSECONDS_PER_SECOND / FRACTIONAL_MAX_TICK_TIME;
const MINIMUM_TICK_MICROSECONDS: u64 = MICROSECONDS_PER_SECOND / FRACTIONAL_MIN_TICK_TIME;
pub const MAXIMUM_TICK_DURATION: Duration = time::Duration::from_micros(MAXIMUM_TICK_MICROSECONDS);
pub const MINIMUM_TICK_DURATION: Duration = time::Duration::from_micros(MINIMUM_TICK_MICROSECONDS);

#[derive(Resource)]
pub struct DeltaTResource {
    pub total_time: Duration,
    pub last_tick: Duration,
    pub last_tick_time: Duration
}

impl DeltaTResource {
    pub fn new() -> Self {
        Self {
            total_time: MINIMUM_TICK_DURATION,
            last_tick: MINIMUM_TICK_DURATION,
            last_tick_time: time::Duration::ZERO
        }
    }
}

pub fn increment_time(mut time: ResMut<DeltaTResource>) {
    let corrected_duration = time.last_tick_time.clamp(MINIMUM_TICK_DURATION, MAXIMUM_TICK_DURATION);
    *time = DeltaTResource {
        total_time: time.total_time + corrected_duration,
        last_tick: corrected_duration,
        last_tick_time: time.last_tick_time
    };
}

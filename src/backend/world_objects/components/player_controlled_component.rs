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

use std::sync::{atomic::AtomicBool, Arc};

use bevy_ecs::component::Component;
use tokio::sync::broadcast;

use crate::connectivity::user_session::ControlInputMessage;

pub struct InputStatus {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub fire: bool
}

impl Default for InputStatus {
    fn default() -> Self {
        Self {forward: false, backward: false, left: false, right: false, fire: false}
    }
}

#[derive(Component)]
pub struct PlayerControlledComponent {
    pub cancel: Arc<AtomicBool>,
    pub input_receiver: broadcast::Receiver<ControlInputMessage>,
    pub input_status: InputStatus
}

impl PlayerControlledComponent {
    pub fn new(receiver: broadcast::Receiver<ControlInputMessage>, cancel: Arc<AtomicBool>) -> Self {
        PlayerControlledComponent{cancel, input_receiver: receiver, input_status: InputStatus::default()}
    }
}
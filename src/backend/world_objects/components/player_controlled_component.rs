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

use bevy_ecs::component::Component;

use crate::backend::data_objects::input_status::InputStatus;

#[derive(Component)]
pub struct PlayerControlledComponent {
        pub input_status: InputStatus
}

impl PlayerControlledComponent {
    pub fn new() -> Self {
        PlayerControlledComponent{input_status: InputStatus::default()}
    }
}
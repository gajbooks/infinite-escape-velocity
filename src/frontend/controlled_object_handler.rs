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

use crate::connectivity::client_server_message::*;
use crate::connectivity::controllable_object_message_data::*;
use macroquad::prelude::*;

pub struct ControlledObjectHandler {
    outgoing_messages: crossbeam_channel::Sender<ClientServerMessage>,
    forward_already_pressed: bool,
    reverse_already_pressed: bool,
    left_already_pressed: bool,
    right_already_pressed: bool,
    fire_already_pressed: bool,
}

impl ControlledObjectHandler {
    pub fn new(
        message_queue: crossbeam_channel::Sender<ClientServerMessage>,
    ) -> ControlledObjectHandler {
        ControlledObjectHandler {
            outgoing_messages: message_queue,
            forward_already_pressed: false,
            reverse_already_pressed: false,
            left_already_pressed: false,
            right_already_pressed: false,
            fire_already_pressed: false,
        }
    }

    pub fn send_updates(&mut self) {
        if is_key_down(KeyCode::Up) {
            if !self.forward_already_pressed {
                self.outgoing_messages.send(
                    ClientServerMessage::ControllableObjectMotionActionForward(
                        ControllableObjectMotionActionData { end_event: false },
                    ),
                );
                self.forward_already_pressed = true;
            }
        } else {
            if self.forward_already_pressed {
                self.outgoing_messages.send(
                    ClientServerMessage::ControllableObjectMotionActionForward(
                        ControllableObjectMotionActionData { end_event: true },
                    ),
                );
                self.forward_already_pressed = false;
            }
        }

        if is_key_down(KeyCode::Down) {
            if !self.reverse_already_pressed {
                self.outgoing_messages.send(
                    ClientServerMessage::ControllableObjectMotionActionReverse(
                        ControllableObjectMotionActionData { end_event: false },
                    ),
                );
                self.reverse_already_pressed = true;
            }
        } else {
            if self.reverse_already_pressed {
                self.outgoing_messages.send(
                    ClientServerMessage::ControllableObjectMotionActionReverse(
                        ControllableObjectMotionActionData { end_event: true },
                    ),
                );
                self.reverse_already_pressed = false;
            }
        }

        if is_key_down(KeyCode::Left) {
            if !self.left_already_pressed {
                self.outgoing_messages.send(
                    ClientServerMessage::ControllableObjectMotionActionLeft(
                        ControllableObjectMotionActionData { end_event: false },
                    ),
                );
                self.left_already_pressed = true;
            }
        } else {
            if self.left_already_pressed {
                self.outgoing_messages.send(
                    ClientServerMessage::ControllableObjectMotionActionLeft(
                        ControllableObjectMotionActionData { end_event: true },
                    ),
                );
                self.left_already_pressed = false;
            }
        }

        if is_key_down(KeyCode::Right) {
            if !self.right_already_pressed {
                self.outgoing_messages.send(
                    ClientServerMessage::ControllableObjectMotionActionRight(
                        ControllableObjectMotionActionData { end_event: false },
                    ),
                );
                self.right_already_pressed = true;
            }
        } else {
            if self.right_already_pressed {
                self.outgoing_messages.send(
                    ClientServerMessage::ControllableObjectMotionActionRight(
                        ControllableObjectMotionActionData { end_event: true },
                    ),
                );
                self.right_already_pressed = false;
            }
        }

        if is_key_down(KeyCode::Space) {
            if !self.fire_already_pressed {
                self.outgoing_messages.send(
                    ClientServerMessage::ControllableObjectMotionActionFire(
                        ControllableObjectMotionActionData { end_event: false },
                    ),
                );
                self.fire_already_pressed = true;
            }
        } else {
            if self.fire_already_pressed {
                self.outgoing_messages.send(
                    ClientServerMessage::ControllableObjectMotionActionFire(
                        ControllableObjectMotionActionData { end_event: true },
                    ),
                );
                self.fire_already_pressed = false;
            }
        }
    }
}

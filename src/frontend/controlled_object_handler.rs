use super::super::connectivity::client_server_message::*;
use super::super::connectivity::controllable_object_message_data::*;
use macroquad::prelude::*;

pub struct ControlledObjectHandler {
    outgoing_messages: crossbeam_channel::Sender<ClientServerMessage>,
    forward_already_pressed: bool,
    reverse_already_pressed: bool,
    left_already_pressed: bool,
    right_already_pressed: bool,
}

impl ControlledObjectHandler {

    pub fn new(message_queue: crossbeam_channel::Sender<ClientServerMessage>) -> ControlledObjectHandler {
        ControlledObjectHandler{outgoing_messages: message_queue, forward_already_pressed: false, reverse_already_pressed: false, left_already_pressed: false, right_already_pressed: false}
    }

    pub fn send_updates(&mut self) {
        if is_key_down(KeyCode::Up) {
            if !self.forward_already_pressed
            {
                self.outgoing_messages.send(ClientServerMessage::ControllableObjectMotionActionForward(ControllableObjectMotionActionData{end_event: false}));
                self.forward_already_pressed = true;
            }
        } else {
            if self.forward_already_pressed {
                self.outgoing_messages.send(ClientServerMessage::ControllableObjectMotionActionForward(ControllableObjectMotionActionData{end_event: true}));
                self.forward_already_pressed = false;
            }
        }

        if is_key_down(KeyCode::Down) {
            if !self.reverse_already_pressed {
                self.outgoing_messages.send(ClientServerMessage::ControllableObjectMotionActionReverse(ControllableObjectMotionActionData{end_event: false}));
                self.reverse_already_pressed = true;
            }
        } else {
            if self.reverse_already_pressed {
                self.outgoing_messages.send(ClientServerMessage::ControllableObjectMotionActionReverse(ControllableObjectMotionActionData{end_event: true}));
                self.reverse_already_pressed = false;
            }
        }

        if is_key_down(KeyCode::Left) {
            if !self.left_already_pressed
            {
                self.outgoing_messages.send(ClientServerMessage::ControllableObjectMotionActionLeft(ControllableObjectMotionActionData{end_event: false}));
                self.left_already_pressed = true;
            }
        } else {
            if self.left_already_pressed {
                self.outgoing_messages.send(ClientServerMessage::ControllableObjectMotionActionLeft(ControllableObjectMotionActionData{end_event: true}));
                self.left_already_pressed = false;
            }
        }

        if is_key_down(KeyCode::Right) {
            if !self.right_already_pressed
            {
                self.outgoing_messages.send(ClientServerMessage::ControllableObjectMotionActionRight(ControllableObjectMotionActionData{end_event: false}));
                self.right_already_pressed = true;
            }
        } else {
            if self.right_already_pressed {
                self.outgoing_messages.send(ClientServerMessage::ControllableObjectMotionActionRight(ControllableObjectMotionActionData{end_event: true}));
                self.right_already_pressed = false;
            }
        }
    }
}
use super::super::connectivity::client_server_message::*;
use super::super::connectivity::server_client_message::*;
use super::super::connectivity::controllable_object_message_data::*;
use super::unique_object_storage::*;
use std::sync::*;
use crossbeam_channel::TryRecvError;
use super::super::shared_types::*;
use super::server_viewport::*;
use super::unique_object::*;
use euclid::*;
use super::collision_component::*;
use super::shape::*;

pub struct PlayerObjectBinding {
    incoming_messages: crossbeam_channel::Receiver<ClientServerMessage>,
    outgoing_messages: crossbeam_channel::Sender<ServerClientMessage>,
    currently_controlled_object: Option<IdType>,
    controllable_objects: Arc<UniqueObjectStorage>,
    player_viewport_id: IdType,
    forward_already_pressed: bool,
    reverse_already_pressed: bool,
    left_already_pressed: bool,
    right_already_pressed: bool,
    fire_already_pressed: bool
}

impl PlayerObjectBinding {
    pub fn new(incoming: crossbeam_channel::Receiver<ClientServerMessage>, outgoing: crossbeam_channel::Sender<ServerClientMessage>,  list: Arc<UniqueObjectStorage>, player_viewport_id: IdType) -> PlayerObjectBinding {
        PlayerObjectBinding{
            incoming_messages: incoming,
            outgoing_messages: outgoing,
            currently_controlled_object: None,
            controllable_objects: list,
            player_viewport_id: player_viewport_id,
            forward_already_pressed: false,
            reverse_already_pressed: false,
            left_already_pressed: false,
            right_already_pressed: false,
            fire_already_pressed: false}
    }
    pub fn handle_updates(&mut self, delta_t: DeltaT) {

        match self.currently_controlled_object {
            Some(_has) => (),
            None => {
                let list = self.controllable_objects.all_objects();
                match list.iter().find(|x|
                    match x.get_type() {
                        ObjectType::WorldObject(_ignored) => true,
                        _ => false
                    }
                ) {
                    Some(exists) => {
                        let id = exists.get_id();
                        self.currently_controlled_object = Some(id);
                        self.outgoing_messages.send(ServerClientMessage::AssignControllableObject(AssignControllableObjectData{id: id}));
                    },
                    None => ()
                }
            }
        };


        loop {
            let message = match self.incoming_messages.try_recv() {
                Ok(has) => has,
                Err(TryRecvError::Empty) => {break;}
                Err(TryRecvError::Disconnected) => {panic!("Server disconnected")}
            };

            match message {
                ClientServerMessage::ControllableObjectMotionActionForward(data) => {
                    if !data.end_event {
                        if !self.reverse_already_pressed
                        {
                            self.forward_already_pressed = true;
                        }
                    } else {
                        self.forward_already_pressed = false;
                    }
                },
                ClientServerMessage::ControllableObjectMotionActionReverse(data) => {
                    if !data.end_event {
                        self.left_already_pressed = false;
                        self.right_already_pressed = false;
                        self.forward_already_pressed = false;
            
                        self.reverse_already_pressed = true;
                    } else {
                        self.reverse_already_pressed = false;
                    }
                },
                ClientServerMessage::ControllableObjectMotionActionLeft(data) => {
                    if !data.end_event {
                        if !self.right_already_pressed
                        {
                            self.left_already_pressed = true;
                        }
                    } else {
                        self.left_already_pressed = false;
                    }
                },
                ClientServerMessage::ControllableObjectMotionActionRight(data) => {
                    if !data.end_event {
                        if !self.left_already_pressed
                        {
                            self.right_already_pressed = true;
                        }
                    } else {
                        self.right_already_pressed = false;
                    }
                },
                ClientServerMessage::ControllableObjectMotionActionFire(data) => {
                    if !data.end_event {
                        self.fire_already_pressed = true;
                    } else {
                        self.fire_already_pressed = true;
                    }
                }
            };
        }

        let controlled: Option<std::sync::Arc<dyn UniqueObject + std::marker::Send + std::marker::Sync>> = match self.currently_controlled_object {
            Some(has) => {
                match self.controllable_objects.get_by_id(has) {
                    Some(object_exists) => {
                        match object_exists.get_type() {
                            ObjectType::WorldObject(_dynamic_object) => Some(object_exists),
                            _ => {
                                self.currently_controlled_object = None;
                                None
                            }
                        }
                    },
                    None => {
                        self.currently_controlled_object = None;
                        None
                    }
                }
            },
            None => None
        };

        match controlled {
            Some(has) => {
                match has.as_motion_component() {
                    Some(motion_component) => {
                        match self.controllable_objects.get_by_id(self.player_viewport_id) {
                            Some(player_viewport) => {
                                match player_viewport.as_collision_component() {
                                    Some(viewport_collision_component) => {
                                        viewport_collision_component.move_center(motion_component.get_coordinates().location);
                                    },
                                    None => {
                                        panic!("Viewport doesn't have a collision component?");
                                    }
                                }
                            }, None => {
                                panic!("Player viewport ded");
                            }
                        }

                        match has.as_controllable_component() {
                            Some(controllable_component) => {
                                if self.left_already_pressed {
                                    controllable_component.turn_left_for_tick();
                                }
        
                                if self.right_already_pressed {
                                    controllable_component.turn_right_for_tick();
                                }
        
                                if self.forward_already_pressed {
                                    controllable_component.accelerate_forward_for_tick();
                                }
        
                                if self.reverse_already_pressed {
                                    let coordinates = motion_component.get_coordinates();
                                    let speed = coordinates.velocity.length();
        
                                    let direction_of_velocity = coordinates.velocity.angle_from_x_axis();
                                    let direction_to_reverse = direction_of_velocity + Angle::pi();
                                    let direction_to_rotate = direction_to_reverse.angle_to(coordinates.rotation);
                                    let float_angle_in_degrees = direction_to_rotate.to_degrees();
        
                                    let angular_veolcity_multiplier = (float_angle_in_degrees.atan()/std::f32::consts::PI).abs() * 1.0;
        
                                    if float_angle_in_degrees < 0.0 {
                                        controllable_component.turn_right_for_tick_with_multiplier(angular_veolcity_multiplier);
                                    }
            
                                    if float_angle_in_degrees > 0.0 {
                                        controllable_component.turn_left_for_tick_with_multiplier(angular_veolcity_multiplier);
                                    }
        
                                    if float_angle_in_degrees.abs() < 1.0 {
                                        if (speed/100.0) > 0.1 {
                                            controllable_component.accelerate_forward_for_tick();
                                        }
                                        else {
                                            motion_component.set_position(None, None, Some(direction_to_reverse.get()));
                                            controllable_component.stop_lateral_motion();
                                        }
                                    }
                                }

                                if self.fire_already_pressed {
                                    controllable_component.fire_for_tick();
                                }
                            }
                            , None => {}
                        }


                    },
                    None => {}
                }
            },
            _ => {}
        }
    }
}
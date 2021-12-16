use super::controllable_object_message_data::*;

pub enum ClientServerMessage {
    ControllableObjectMotionActionForward(ControllableObjectMotionActionData),
    ControllableObjectMotionActionReverse(ControllableObjectMotionActionData),
    ControllableObjectMotionActionLeft(ControllableObjectMotionActionData),
    ControllableObjectMotionActionRight(ControllableObjectMotionActionData),
    ControllableObjectMotionActionFire(ControllableObjectMotionActionData)
}
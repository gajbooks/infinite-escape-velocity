use super::dynamic_object_message_data::*;

pub enum ServerClientMessage {
    DynamicObjectUpdate(DynamicObjectMessageData),
    DynamicObjectCreation(DynamicObjectCreationData),
    DynamicObjectDestruction(DynamicObjectDestructionData)
}
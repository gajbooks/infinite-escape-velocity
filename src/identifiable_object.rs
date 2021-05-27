use std::sync::atomic::*;

pub type IdType = u32;
pub type AtomicIdType = AtomicU32;

#[derive(Clone)]
pub struct SentFrom<T: Clone> {
    pub origin_id: IdType,
    pub data: T
}

#[derive(Clone)]
pub struct SendTo<T: Clone> {
    pub destination_id: IdType,
    pub from: SentFrom<T>
}
use std::{sync::{atomic::{AtomicBool, Ordering}, Arc}, net::SocketAddr};

use euclid::{Point2D, Length};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{connectivity::{server_client_message::ServerClientMessage, client_server_message::ClientServerMessage}, backend::{world_objects::server_viewport::ServerViewport, world_object_storage::{ephemeral_id_allocator::IdAllocatorType, world_object::WorldObject}, shape::{CircleData, Shape}}};

pub struct UserSession {
    pub remote_address: SocketAddr,
    cancel: Arc<AtomicBool>,
    viewport: Arc<ServerViewport>,
    dead: AtomicBool
}

impl UserSession {
    pub fn new(
        to_remote: UnboundedSender<ServerClientMessage>,
        from_remote: UnboundedReceiver<ClientServerMessage>,
        remote_address: SocketAddr,
        cancel: Arc<AtomicBool>,
        id_generator: IdAllocatorType
    ) -> Arc<UserSession> {
        let session = Arc::new(UserSession {
            remote_address: remote_address,
            cancel: cancel,
            dead: false.into(),
            viewport: Arc::new(ServerViewport::new(Shape::Circle(CircleData{location: Point2D::new(0.0, 0.0), radius: Length::new(500.0)}), id_generator, to_remote.clone()))
        });
        tokio::spawn(session.clone().process_incoming_messages(to_remote, from_remote));
        session
    }

    async fn process_incoming_messages(self: Arc<Self>, to_remote: UnboundedSender<ServerClientMessage>, mut from_remote: UnboundedReceiver<ClientServerMessage>,) {
        loop {
            if self.cancel.load(Ordering::Relaxed) == true {
                break;
            }

            let message = from_remote.recv().await;

            let message = match message {
                Some(x) => x,
                None => {
                    continue;
                }
            };

            match message {
                ClientServerMessage::Disconnect => {
                    self.disconnect();
                    continue;
                },
                _ => ()
            }
        }

        self.mark_dead();
    }

    fn mark_dead(&self) {
        self.dead.store(true, Ordering::Relaxed);
    }

    pub fn is_dead(&self) -> bool {
        self.dead.load(Ordering::Relaxed)
    }

    pub fn disconnect(&self) {
        let _ = self.cancel.store(true, Ordering::Relaxed);
    }

    pub fn get_viewport(&self) -> Arc<dyn WorldObject> {
        return self.viewport.clone();
    }
}
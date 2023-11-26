use std::{sync::{atomic::{AtomicBool, Ordering}, Arc}, net::SocketAddr};

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::connectivity::{server_client_message::ServerClientMessage, client_server_message::ClientServerMessage};

pub struct UserSession {
    pub remote_address: SocketAddr,
    cancel: Arc<AtomicBool>,
    dead: AtomicBool
}

impl UserSession {
    pub fn new(
        to_remote: UnboundedSender<ServerClientMessage>,
        from_remote: UnboundedReceiver<ClientServerMessage>,
        remote_address: SocketAddr,
        cancel: Arc<AtomicBool>,
    ) -> Arc<UserSession> {
        let session = Arc::new(UserSession {
            remote_address: remote_address,
            cancel: cancel,
            dead: false.into()
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
}
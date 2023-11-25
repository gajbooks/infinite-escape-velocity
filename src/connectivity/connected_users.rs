use std::{sync::{Mutex, Arc}, time::Duration};

use tokio::time;

use crate::connectivity::user_session::UserSession;

pub struct ConnectedUsers {
    connection_list: Mutex<Vec<Arc<UserSession>>>,
}

impl ConnectedUsers {
    pub fn new() -> ConnectedUsers {
        ConnectedUsers { connection_list: Mutex::new(Vec::new()) }
    }

    pub fn add_user(&self, new_user: Arc<UserSession>) {
        let mut connected_users = self.connection_list.lock().unwrap();
        println!("New user connected from {}", new_user.remote_address);
        connected_users.push(new_user);
    }

    pub async fn garbage_collector(connected_users: Arc<ConnectedUsers>) {
        let mut interval = time::interval(Duration::from_millis(500));

        loop {
            interval.tick().await;
            {
                let mut list = connected_users.connection_list.lock().unwrap();
                list.retain(|x| !x.is_dead());
            }
        }
    }
}
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

use tokio::sync::broadcast::{self, Receiver, Sender};
use tracing::info;

use crate::connectivity::models::chat_message_response::ChatMessageResponse;

const MESSAGE_QUEUE_CAPACITY: usize = 100;

#[derive(Clone)]
pub struct ChatService {
    broadcast: Sender<ChatMessageResponse>,
}

impl Default for ChatService {
    fn default() -> Self {
        let (sender, _) = broadcast::channel(MESSAGE_QUEUE_CAPACITY);
        Self { broadcast: sender }
    }
}

impl ChatService {
    pub fn send_message(&self, message: &str, player_name: Option<&str>) {
        let message = ChatMessageResponse {
            message: message.to_owned(),
            player_name: player_name.map(|x| x.to_owned()),
        };

        info!("[CHAT] {}", message.to_string());

        // If there are no handles, we don't care about dropping the message or doing anything
        let _ = self.broadcast.send(message);
    }

    pub fn get_receiving_handle(&self) -> Receiver<ChatMessageResponse> {
        self.broadcast.subscribe()
    }
}

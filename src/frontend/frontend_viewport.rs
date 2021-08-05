use super::super::shared_types::*;
use crossbeam_channel::TryRecvError;
use dashmap::DashMap;
use fxhash::FxBuildHasher;
use macroquad::prelude::*;
use rayon::prelude::*;
use super::super::connectivity::server_client_message::*;
use super::dynamic_object_client_data::*;

pub struct FrontendViewport {
    incoming_messages: crossbeam_channel::Receiver<ServerClientMessage>,
    lag_compensation_cache: DashMap<IdType, DynamicObjectClientData, FxBuildHasher>
}

impl FrontendViewport {
    pub fn new(incoming_queue: crossbeam_channel::Receiver<ServerClientMessage>) -> FrontendViewport {
        return FrontendViewport{incoming_messages: incoming_queue, lag_compensation_cache: DashMap::with_hasher(FxBuildHasher::default())}
    }

    pub async fn tick(&self, delta_t: f32) {
        self.lag_compensation_cache.par_iter_mut().for_each(|mut x| {
            let mut value = x.value_mut();
            value.x += (value.vx * delta_t) as f64;
            value.y += (value.vy * delta_t) as f64;
            value.rotation += value.angular_velocity * delta_t;
        });

        loop {
            let message = match self.incoming_messages.try_recv() {
                Ok(has) => has,
                Err(TryRecvError::Empty) => {break;}
                Err(TryRecvError::Disconnected) => {panic!("Server disconnected")}
            };

            match message {
                ServerClientMessage::DynamicObjectCreation(created) => {
                    self.lag_compensation_cache.insert(created.id, DynamicObjectClientData{
                        x: 0.0, y: 0.0, rotation: 0.0, vx: 0.0, vy: 0.0, angular_velocity: 0.0, object_type: 0});
                },
                ServerClientMessage::DynamicObjectDestruction(deleted) => {
                    self.lag_compensation_cache.remove(&deleted.id);
                },
                ServerClientMessage::DynamicObjectUpdate(update) => {
                    match self.lag_compensation_cache.entry(update.id) {
                        dashmap::mapref::entry::Entry::Vacant(_vacant) => (),
                        dashmap::mapref::entry::Entry::Occupied(has) => {
                            has.replace_entry(DynamicObjectClientData{x: update.x, y: update.y, rotation: update.rotation, vx: update.vx, vy: update.vy, angular_velocity: update.angular_velocity, object_type: update.object_type});
                        }
                    }
                }
            }
        }

        self.render().await;
    }

    async fn render(&self) {
        clear_background(BLACK);

        for x in &self.lag_compensation_cache {
            draw_circle((screen_width() / 2.0) + x.value().x as f32, (screen_height() / 2.0) + x.value().y as f32, 20.0, YELLOW);
        }
        next_frame().await;
    }
}
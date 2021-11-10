use super::super::shared_types::*;
use crossbeam_channel::TryRecvError;
use dashmap::DashMap;
use fxhash::FxBuildHasher;
use macroquad::prelude::*;
use rayon::prelude::*;
use super::super::connectivity::server_client_message::*;
use super::dynamic_object_client_data::*;
use super::object_texture_mapping::*;

enum CameraFollow {
    ObjectId(IdType),
    Coordinates(Coordinates)
}

pub struct FrontendViewport {
    incoming_messages: crossbeam_channel::Receiver<ServerClientMessage>,
    lag_compensation_cache: DashMap<IdType, DynamicObjectClientData, FxBuildHasher>,
    object_index: ObjectIndex,
    camera_follow: CameraFollow,
    last_coordinates: Coordinates,
}

impl FrontendViewport {
    pub fn new(incoming_queue: crossbeam_channel::Receiver<ServerClientMessage>, object_index: ObjectIndex) -> FrontendViewport {
        return FrontendViewport{incoming_messages: incoming_queue, lag_compensation_cache: DashMap::with_hasher(FxBuildHasher::default()), object_index: object_index, camera_follow: CameraFollow::Coordinates(Coordinates{x: 0.0, y: 0.0}), last_coordinates: Coordinates{x:0.0, y:0.0}}
    }

    pub async fn tick(&mut self, delta_t: f32) {

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
                        x: 0.0, y: 0.0, rotation: 0.0, vx: 0.0, vy: 0.0, angular_velocity: 0.0, object_type: ObjectType::NonWorld(), texture: None});
                },
                ServerClientMessage::DynamicObjectDestruction(deleted) => {
                    self.lag_compensation_cache.remove(&deleted.id);
                },
                ServerClientMessage::DynamicObjectUpdate(update) => {
                    match self.lag_compensation_cache.entry(update.id) {
                        dashmap::mapref::entry::Entry::Vacant(_vacant) => (),
                        dashmap::mapref::entry::Entry::Occupied(has) => {

                            let texture = match &has.get().texture {
                                Some(has) => {
                                    if has.verify_type(&update.object_type) {
                                        has.to_owned()
                                    } else {
                                        self.object_index.map_object_type_to_texture(&update.object_type)
                                    }
                                },
                                None => {
                                    self.object_index.map_object_type_to_texture(&update.object_type)
                                }
                            };

                            has.replace_entry(DynamicObjectClientData{x: update.x, y: update.y, rotation: update.rotation, vx: update.vx, vy: update.vy, angular_velocity: update.angular_velocity, object_type: update.object_type, texture: Some(texture)});
                        }
                    }
                },
                ServerClientMessage::AssignControllableObject(assigned) => {
                    self.camera_follow = CameraFollow::ObjectId(assigned.id);
                }
            }
        }

        self.render().await;
    }

    async fn render(&mut self) {
        clear_background(BLACK);

        let camera_coordinates = match &self.camera_follow {
            CameraFollow::Coordinates(camera_coordinates) => camera_coordinates,
            CameraFollow::ObjectId(id) => {
                match self.lag_compensation_cache.get(&id) {
                    Some(has) => {
                        let center_x = has.x - (screen_width() / 2.0) as f64;
                        let center_y = has.y - (screen_height() / 2.0) as f64;
                        self.last_coordinates = Coordinates{x: center_x, y: center_y};
                        &self.last_coordinates
                    },
                    None => {
                        self.camera_follow = CameraFollow::Coordinates(self.last_coordinates.clone());
                        &self.last_coordinates
                    }
                }
            }
        };

        for object in &self.lag_compensation_cache {
            match &object.value().texture {
                Some(texture) => {
                    let params = DrawTextureParams{dest_size: None, source: None, rotation: object.value().rotation, flip_x: false, flip_y: false, pivot: None};
                    let texture = *texture.get_texture();
                    draw_texture_ex(texture, (object.value().x - camera_coordinates.x) as f32 - (texture.width() / 2.0), (object.value().y - camera_coordinates.y) as f32 - (texture.height() / 2.0), WHITE, params);
                },
                None => ()
            };
        }
        next_frame().await;
    }
}
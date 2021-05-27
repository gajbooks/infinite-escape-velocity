mod spatial_hashmap;
mod distributing_queue;
mod identifiable_object;
mod shape;
mod aabb_iterator;
mod hash_coordinates;
mod shrink_storage;
mod unique_id_allocator;
mod unique_object_storage;
use shape::*;
use spatial_hashmap::*;
use std::sync::*;
use identifiable_object::*;
use std::thread;
use unique_id_allocator::*;
use unique_object_storage::*;

struct CollidingObject {
    collision_query_sender: crossbeam_channel::Sender<SpatialMessage>,
    collision_query_receiver: crossbeam_channel::Receiver<SentFrom<Arc<Shape>>>,
    id: ReturnableId,
    position: Arc<Shape>
}

impl StoredObject for CollidingObject {
    fn get_id(&self) -> IdType {
        return self.id.id;
    }

    fn process_messages(&self) {
        self.print_received();
    }
}

impl CollidingObject {

    pub fn new(spatial_map: &spatial_hashmap::SpatialHashmap, spatial_response_distributor: &distributing_queue::DistributingQueue<Arc<Shape>>, position: Shape, id: ReturnableId) -> CollidingObject {
        let (s, r) = crossbeam_channel::unbounded();
        spatial_response_distributor.register_receiver(id.id, s);
        let spatial_map_channel = spatial_map.get_channel();
        let position = Arc::new(position);
        return CollidingObject {collision_query_sender: spatial_map_channel, collision_query_receiver: r, id: id, position: position};
    }

    pub fn print_received(&self) {
        self.collision_query_sender.send(SpatialMessage::Add{new: SentFrom{origin_id: self.id.id, data: self.position.clone()}});
        match self.collision_query_receiver.try_recv() {
            Ok(mesg) => {
                println!("From ID: {}, Shape type: {:?}", mesg.origin_id, mesg.data);
            },
            Err(_e) => {
                // Who effin cares
            }
        }
    }
}

fn main() {
    let spatial_response_queue = distributing_queue::DistributingQueue::<Arc<Shape>>::new();
    let map = spatial_hashmap::SpatialHashmap::new(spatial_response_queue.get_sender());
    let storage = UniqueObjectStorage::new();
    let unique_id_generator = UniqueIdAllocator::new();
    storage.add(Box::new(CollidingObject::new(&map, &spatial_response_queue, Shape::RoundedTube(RoundedTubeData{x1: -1.0, y1: 2.0, x2: 1.0, y2: 2.0, r:1.0}), unique_id_generator.new_allocated_id())));
    storage.add(Box::new(CollidingObject::new(&map, &spatial_response_queue, Shape::RoundedTube(RoundedTubeData{x1: -1.0, y1: 1.0, x2: 1.0, y2: 1.0, r:1.0}), unique_id_generator.new_allocated_id())));

    let mut threads = vec![];

    threads.push(thread::spawn(move || {
        loop {
            map.process_entry();
        }
    }));

    threads.push(thread::spawn(move || {
        loop {
            spatial_response_queue.process_queue();
        }
    }));

    threads.push(thread::spawn(move || {
        loop {
            storage.process_object_messages();
        }
    }));

    for child in threads {
        let _ = child.join();
    }
}

use std::sync::{Arc, atomic::AtomicUsize};

use dashmap::DashMap;

use crate::room::{Room, RoomId};

#[derive(Debug)]
pub struct ActiveConnectionGuard(Arc<AtomicUsize>);

impl Drop for ActiveConnectionGuard {
    fn drop(&mut self) {
        tracing::debug!("decrementing active connections");
        self.0.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }
}

impl ActiveConnectionGuard {
    pub fn new(active_connections: &Arc<AtomicUsize>) -> Self {
        tracing::debug!("incrementing active connections");
        active_connections.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Self(active_connections.clone())
    }
}

#[derive(Debug)]
pub struct GlobalState {
    active_connections: Arc<AtomicUsize>,
    rooms: Arc<DashMap<RoomId, Room>>,
}

impl Default for GlobalState {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalState {
    pub fn new() -> Self {
        tracing::debug!("creating global state");

        let rooms = Arc::new(DashMap::new());
        for (id, room) in Room::default_public() {
            rooms.insert(id, room);
        }

        Self {
            active_connections: Arc::new(AtomicUsize::new(0)),
            rooms,
        }
    }

    // pub fn inc_active_connections(&self) {
    //     tracing::debug!("incrementing active connections");
    //     self.active_connections
    //         .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    // }

    // pub fn dec_active_connections(&self) {
    //     tracing::debug!("decrementing active connections");
    //     self.active_connections
    //         .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    // }

    pub fn get_active_connections(&self) -> usize {
        self.active_connections
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn active_connection_guard(&self) -> ActiveConnectionGuard {
        ActiveConnectionGuard::new(&self.active_connections)
    }

    pub fn get_rooms(&self) -> Vec<Room> {
        let mut rooms: Vec<_> = self
            .rooms
            .iter()
            .map(|r| r.value().clone())
            .filter(|r| r.is_public)
            .collect();
        // index is always some for public DEFAULT rooms
        rooms.sort_by(|a, b| a.index.unwrap().cmp(&b.index.unwrap()));
        rooms
    }

    pub fn get_room(&self, id: RoomId) -> Option<Room> {
        self.rooms.get(&id).map(|v| v.value().clone())
    }

    pub fn insert_room(&self, room: Room) -> Room {
        let id = room.id;
        match self.rooms.insert(id, room) {
            Some(_) => panic!("room id shouldn't be collide with existing one"),
            None => self.rooms.get(&id).unwrap().value().clone(),
        }
    }
}

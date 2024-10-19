use dashmap::DashMap;
use std::cmp::Ordering;
use std::{collections::HashSet, sync::Arc};
use tokio::sync::broadcast::{self, Sender};

const ROOM_CHANNEL_CAPACITY: usize = 1024;
pub const DEFAULT_ROOM: &str = "lobby";

#[derive(Clone)]
pub enum RoomMsg {
    Joined(String),
    Left(String),
    Msg(Arc<str>),
}

pub struct Room {
    tx: Sender<RoomMsg>,
    users: HashSet<String>,
}

impl Room {
    fn new() -> Self {
        let (tx, _) = broadcast::channel(ROOM_CHANNEL_CAPACITY);
        let users = HashSet::with_capacity(8);
        Self { tx, users }
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct Rooms(Arc<DashMap<String, Room>>);

impl Rooms {
    pub fn new() -> Self {
        Self(Arc::new(DashMap::with_capacity(8)))
    }

    pub fn join(&self, room_name: &str, user_name: &str) -> Sender<RoomMsg> {
        let mut room = self.0.entry(room_name.into()).or_insert(Room::new());
        room.users.insert(user_name.into());
        room.tx.clone()
    }

    pub fn leave(&self, room_name: &str, user_name: &str) {
        let mut delete_room = false;
        if let Some(mut room) = self.0.get_mut(room_name) {
            room.users.remove(user_name);
            delete_room = room.tx.receiver_count() <= 1;
        }
        if delete_room {
            self.0.remove(room_name);
        }
    }

    pub fn change(&self, prev_room: &str, next_room: &str, user_name: &str) -> Sender<RoomMsg> {
        self.leave(prev_room, user_name);
        self.join(next_room, user_name)
    }

    pub fn change_name(&self, room_name: &str, prev_name: &str, new_name: &str) {
        if let Some(mut room) = self.0.get_mut(room_name) {
            room.users.remove(prev_name);
            room.users.insert(String::from(new_name));
        }
    }

    pub fn list(&self) -> Vec<(String, usize)> {
        let mut list: Vec<_> = self
            .0
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().tx.receiver_count()))
            .collect();
        list.sort_by(|a, b| match b.1.cmp(&a.1) {
            Ordering::Equal => a.0.cmp(&b.0),
            ordering => ordering,
        });
        list
    }

    pub fn list_users(&self, room_name: &str) -> Option<Vec<String>> {
        self.0.get(room_name).map(|room| {
            let mut users = room.users.iter().cloned().collect::<Vec<_>>();
            users.sort();
            users
        })
    }
}

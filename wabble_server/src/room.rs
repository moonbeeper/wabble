use std::{
    ops::Deref,
    sync::{Arc, atomic::AtomicUsize},
};

use rand::Rng;
use tokio::sync::broadcast;

const ROOM_MAX_CONNECTIONS: usize = 32;

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, std::hash::Hash)]
// pub enum RoomId {
//     Public(uuid::Uuid),
//     Private(uuid::Uuid),
// }

// impl RoomId {
//     pub fn default_public() -> Vec<Self> {
//         vec![
//             Self::Public(uuid::uuid!("399e9bd9-6fab-4492-8d31-268e1fd4f34a")),
//             Self::Public(uuid::uuid!("fe34f9a2-2782-4b42-9101-2b0fbea9e9a8")),
//             Self::Public(uuid::uuid!("09e329ae-cbac-448e-a6ef-460575640a35")),
//             Self::Public(uuid::uuid!("6ffbbccf-994f-4dd7-90ca-e2fadd027edd")),
//         ]
//     }

//     pub fn id(&self) -> uuid::Uuid {
//         match self {
//             Self::Private(v) | Self::Public(v) => *v,
//         }
//     }
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, std::hash::Hash)]
pub struct RoomId(uuid::Uuid);

impl RoomId {
    pub fn default_public() -> Vec<Self> {
        vec![
            Self(uuid::uuid!("399e9bd9-6fab-4492-8d31-268e1fd4f34a")),
            Self(uuid::uuid!("fe34f9a2-2782-4b42-9101-2b0fbea9e9a8")),
            Self(uuid::uuid!("09e329ae-cbac-448e-a6ef-460575640a35")),
            Self(uuid::uuid!("6ffbbccf-994f-4dd7-90ca-e2fadd027edd")),
        ]
    }

    pub fn id(&self) -> uuid::Uuid {
        self.0
    }
}

impl From<uuid::Uuid> for RoomId {
    fn from(value: uuid::Uuid) -> Self {
        Self(value)
    }
}

pub type RoomTx = broadcast::Sender<RoomMessage>;
pub type RoomRx = broadcast::Receiver<RoomMessage>;

#[derive(Debug, Clone)]
pub struct Room {
    pub id: uuid::Uuid,
    pub name: String, // generated only for public rooms, not for private ones. practically for show
    pub active_connections: Arc<AtomicUsize>,
    pub tx: RoomTx,
    pub max_connections: usize,
    pub is_public: bool,
    pub index: Option<usize>, // only for public rooms, indicates the order to display them lmao
}

#[derive(Debug)]
pub struct Persona {
    pub name: String,
    pub color: String, // hex rrggbbaa
}

impl Default for Persona {
    fn default() -> Self {
        let numbers: Vec<String> = (0..12)
            .map(|_| rand::rng().random_range(0..9))
            .map(|n: u8| n.to_string())
            .collect();
        let name = format!("user{}", numbers.join(""));
        let color = random_color::RandomColor {
            luminosity: Some(random_color::options::Luminosity::Light),
            ..Default::default()
        }
        .to_rgb_array();

        Self {
            name,
            color: format!("0x{:02X}{:02X}{:02X}FF", color[0], color[1], color[2],),
        }
    }
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct MessagePersona {
    pub id: uuid::Uuid, // differentiate users
    pub username: String,
    //differentiate users if a use shares the same name by giving them a different color without letting them know
    pub color: String, // hex rrggbbaa
}

impl MessagePersona {
    pub fn from_persona(id: uuid::Uuid, persona: &Persona) -> Self {
        Self {
            id,
            username: persona.name.clone(),
            color: persona.color.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RoomMessage {
    pub persona: MessagePersona,
    pub message: String, // client formats the message into lines
}

impl RoomMessage {
    pub fn system(message: String) -> Self {
        Self {
            persona: MessagePersona {
                id: uuid::Uuid::nil(),
                username: "System".to_string(),
                color: "0xEDA728FF".to_string(),
            },
            message,
        }
    }
}

impl Room {
    pub fn new(id: uuid::Uuid, name: String, is_public: bool, index: Option<usize>) -> Self {
        let (tx, _rx) = broadcast::channel(ROOM_MAX_CONNECTIONS); // hardcoded max
        Self {
            id,
            name,
            active_connections: Arc::new(AtomicUsize::new(0)),
            tx,
            max_connections: ROOM_MAX_CONNECTIONS,
            is_public,
            index,
        }
    }

    pub fn subscribe(&self) -> Option<RoomRx> {
        if self.current_connections() >= ROOM_MAX_CONNECTIONS {
            None
        } else {
            self.active_connections
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            Some(self.tx.subscribe())
        }
    }

    pub fn dec_active_connections(&self) {
        if self.current_connections() == 0 {
            return;
        }

        self.active_connections
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn current_connections(&self) -> usize {
        self.active_connections
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn default_public() -> Vec<(RoomId, Self)> {
        let room_ids = RoomId::default_public();
        let mut rooms = Vec::new();
        for (i, id) in room_ids.iter().enumerate() {
            let name = format!("Public Room {}", i + 1);
            let room = Room::new(id.id(), name, true, Some(i));
            rooms.push((*id, room))
        }

        rooms
    }
}

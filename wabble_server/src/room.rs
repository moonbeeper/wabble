use std::{
    str::FromStr,
    sync::{Arc, atomic::AtomicUsize},
};

use rand::Rng;
use tokio::sync::broadcast;

const ROOM_MAX_CONNECTIONS: usize = 32;

macro_rules! ttid {
    ($ttid:expr) => {{
        match mtid::Ttid::from_str($ttid) {
            Ok(u) => u,
            Err(_) => panic!("invalid Ttid"),
        }
    }};
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, std::hash::Hash)]
pub struct RoomId(mtid::Ttid);

impl Default for RoomId {
    fn default() -> Self {
        Self::new()
    }
}

impl RoomId {
    pub fn default_public() -> Vec<Self> {
        vec![
            Self(ttid!("0vt-5aw-m0y")),
            Self(ttid!("njj-67c-hjx")),
            Self(ttid!("x95-2jt-697")),
            Self(ttid!("3q5-2wc-332")),
        ]
    }

    pub fn id(&self) -> mtid::Ttid {
        self.0
    }

    pub fn new() -> Self {
        Self(mtid::Ttid::random())
    }
}

impl From<mtid::Ttid> for RoomId {
    fn from(value: mtid::Ttid) -> Self {
        Self(value)
    }
}

pub type RoomTx = broadcast::Sender<RoomMessage>;
pub type RoomRx = broadcast::Receiver<RoomMessage>;

#[derive(Debug)]
pub struct RoomSubscription {
    pub rx: RoomRx,
    pub room: Room,
}

impl Drop for RoomSubscription {
    fn drop(&mut self) {
        tracing::debug!(
            "decrementing current active connections for room {}",
            self.room.id.id()
        );

        self.room.dec_active_connections();
    }
}

impl RoomSubscription {
    pub fn send(
        &self,
        message: RoomMessage,
    ) -> Result<usize, broadcast::error::SendError<RoomMessage>> {
        self.room.tx.send(message)
    }

    pub async fn recv(&mut self) -> Result<RoomMessage, broadcast::error::RecvError> {
        self.rx.recv().await
    }

    pub async fn send_hello(&mut self, persona: &Persona) {
        match self.send(RoomMessage::system(format!(
            "{} joined the room",
            persona.name
        ))) {
            Ok(_) => {}
            Err(broadcast::error::SendError(_)) => {
                tracing::error!("failed to send hello message to room {}", self.room.id.id());
            }
        }
    }

    pub async fn send_bye(&mut self, persona: &Persona) {
        let _ = self.send(RoomMessage::system(format!(
            "{} left the room",
            persona.name
        )));
    }
}

#[derive(Debug, Clone)]
pub struct Room {
    pub id: RoomId,
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
    pub fn new(id: RoomId, name: String, is_public: bool, index: Option<usize>) -> Self {
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

    pub fn new_private() -> Self {
        let id = RoomId::new();
        let name = format!("Private Room {}", &id.id().to_string()[..4]);
        Self::new(id, name, false, None)
    }

    pub fn subscribe(&self) -> Option<RoomSubscription> {
        if self.current_connections() >= ROOM_MAX_CONNECTIONS {
            None
        } else {
            self.active_connections
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            Some(RoomSubscription {
                room: self.clone(),
                rx: self.tx.subscribe(),
            })
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
            let room = Room::new(*id, name, true, Some(i));
            rooms.push((*id, room))
        }

        rooms
    }
}

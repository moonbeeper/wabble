use std::{
    str::FromStr,
    sync::{Arc, Mutex, atomic::AtomicUsize},
};

use rand::Rng;
use tokio::sync::broadcast;

use crate::responses;

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
    pub persona: Arc<Mutex<Persona>>,
}

impl Drop for RoomSubscription {
    fn drop(&mut self) {
        tracing::debug!(
            "decrementing current active connections for room {}",
            self.room.id.id()
        );

        self.room.dec_active_connections();

        let mut personas = self.room.personas.lock().unwrap();
        let persona_id = self.persona.lock().unwrap().id;

        personas.retain(|v| {
            let v_id = v.lock().unwrap().id;
            v_id != persona_id
        });
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

    pub async fn send_invite(&mut self, room_id: RoomId) {
        let _ = self.send(RoomMessage::system(format!(
            "Created a new room! Your room code is {} !",
            room_id.id()
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
    pub personas: Arc<Mutex<Vec<Arc<Mutex<Persona>>>>>, // oh god WHAT HAVE I DONE
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Persona {
    pub id: uuid::Uuid,
    pub name: String,
    pub color: String, // hex
    pub forced_color: Option<String>,
}

impl Persona {
    pub fn random_color() -> String {
        let color = random_color::RandomColor {
            luminosity: Some(random_color::options::Luminosity::Light),
            ..Default::default()
        }
        .to_rgb_array();
        format!("0x{:02X}{:02X}{:02X}FF", color[0], color[1], color[2])
    }
    pub fn new(id: uuid::Uuid) -> Self {
        let numbers: Vec<String> = (0..12)
            .map(|_| rand::rng().random_range(0..9))
            .map(|n: u8| n.to_string())
            .collect();
        let name = format!("user{}", numbers.join(""));

        Self {
            id,
            name,
            color: Persona::random_color(),
            forced_color: None,
        }
    }

    pub fn from_response(response: responses::Persona, id: uuid::Uuid) -> Self {
        Self {
            id,
            name: response.name.clone(),
            color: response.color.clone(),
            forced_color: None,
        }
    }
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct MessagePersona {
    pub id: uuid::Uuid, // differentiate users
    pub name: String,
    //differentiate users if a use shares the same name by giving them a different color without letting them know
    pub color: String, // hex rrggbbaa
}

impl MessagePersona {
    pub fn from_persona(persona: &Persona) -> Self {
        println!("forced color: {:?}", persona.forced_color);
        Self {
            id: persona.id,
            name: persona.name.clone(),
            color: persona
                .forced_color
                .as_ref()
                .unwrap_or(&persona.color)
                .clone(),
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
                name: "System".to_string(),
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
            personas: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn new_private() -> Self {
        let id = RoomId::new();
        let name = format!("Private Room {}", &id.id().to_string()[..4]);
        Self::new(id, name, false, None)
    }

    pub async fn subscribe(&self, persona: Arc<Mutex<Persona>>) -> Option<RoomSubscription> {
        if self.current_connections() >= ROOM_MAX_CONNECTIONS {
            None
        } else {
            self.active_connections
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            let mut personas = self
                .personas
                .try_lock()
                .expect("couldn't lock room's personas");

            let name = persona
                .try_lock()
                .expect("failed to lock persona's name")
                .name
                .clone();
            let collisions = personas
                .iter()
                .any(|v| v.try_lock().expect("failed to lock room's persona").name == name);

            if collisions {
                persona.lock().unwrap().forced_color = Some(Persona::random_color());
            }

            personas.push(persona.clone());

            Some(RoomSubscription {
                room: self.clone(),
                rx: self.tx.subscribe(),
                persona,
            })
        }
    }

    pub fn check_collisions(&self, persona: Arc<Mutex<Persona>>) {
        let personas = self
            .personas
            .try_lock()
            .expect("couldn't lock room's personas");

        let this_persona = persona
            .try_lock()
            .expect("failed to lock persona's name")
            .clone();
        let has_collision = personas
            .iter()
            .filter(|v| v.try_lock().expect("failed to lock persona").id != this_persona.id) // Exclude self
            .any(|v| v.try_lock().expect("failed to lock persona").name == this_persona.name);

        let mut persona_guard = persona.try_lock().expect("failed to lock persona's name");
        if has_collision {
            tracing::debug!("collision found, adding forced color");
            persona_guard.forced_color = Some(Persona::random_color());
        } else {
            tracing::debug!("no collision found, resetting forced color");

            persona_guard.forced_color = None;
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

use std::sync::Arc;

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{self, WebSocket},
    },
    response::IntoResponse,
};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::{
    global::{ActiveConnectionGuard, GlobalState},
    responses::{self, Opcode, SocketComms, SocketResponse},
    room::{MessagePersona, Persona, Room, RoomMessage, RoomSubscription},
};

#[derive(Debug)]
struct SocketConnection {
    id: Uuid,
    socket: WebSocket,
    persona: Persona,
    global: Arc<GlobalState>,
    _guard: ActiveConnectionGuard,
    room_subscription: Option<RoomSubscription>,
}

impl SocketConnection {
    fn new(socket: WebSocket, guard: ActiveConnectionGuard, global: Arc<GlobalState>) -> Self {
        Self {
            id: Uuid::new_v4(),
            socket,
            persona: Persona::default(),
            global,
            _guard: guard,
            room_subscription: None,
        }
    }

    async fn send(&mut self, data: impl SocketResponse + serde::Serialize) {
        self.socket
            .send(SocketComms::new(data).into())
            .await
            .expect("failed to send message to socket");
    }

    async fn serve(&mut self) {
        // TODO: handle serde and other errors by using the typical enum pattern
        self.send(responses::Handshake {
            session_id: self.id,
            heartbeat_interval: 30,
            active_connections: self.global.get_active_connections(),
            public_rooms: self.global.get_rooms().iter().map(|r| r.into()).collect(),
        })
        .await;

        loop {
            tokio::select! {
                // Since `ws` is a `Stream`, it is by nature cancel-safe.
                res = self.socket.recv() => {
                    match res {
                        Some(Ok(ws::Message::Text(s))) => {
                            let data = serde_json::from_str::<SocketComms>(s.as_str()).expect("failed parsing incoming data");
                            tracing::debug!("received message: {:#?}", data);
                            self.handle_message(data).await;
                        }
                        Some(Ok(_)) => {}
                        Some(Err(e)) => {
                            tracing::debug!("client disconnected abruptly: {e}");
                            self.leave_room().await;
                        },
                        None => {
                            tracing::debug!("client disconnected gracefully");
                            self.leave_room().await;
                            break;
                        },
                    }
                }
                msg = async {
                    match &mut self.room_subscription {
                        Some(rx) => rx.recv().await,
                        None => std::future::pending().await, // never resolves
                    }
                } => {
                    match msg {
                        Ok(broadcast_msg) => {
                            tracing::debug!("broadcasting message to socket {}: {:?}", self.id, broadcast_msg);

                            if broadcast_msg.persona.id == self.id {
                                tracing::debug!("skipping echo message to self for socket {}", self.id);
                                continue;
                            }

                            self.send(responses::EchoMessage::from(broadcast_msg)).await;
                        }
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            tracing::warn!("socket {} lagged and skipped {} messages", self.id, skipped);
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            tracing::debug!("room broadcast channel closed for socket {}", self.id);
                            self.leave_room().await;
                        }
                    }
                }

            }
        }
    }

    async fn handle_message(&mut self, data: SocketComms) {
        match data.opcode {
            Opcode::Persona => {
                let persona: responses::Persona =
                    serde_json::from_value(data.data).expect("failed parsing persona");

                tracing::debug!("received new persona");
                self.persona = persona.into();
                tracing::debug!("updated persona: {:#?}", self.persona);
            }
            Opcode::JoinRoom => {
                let room: responses::JoinRoom =
                    serde_json::from_value(data.data).expect("failed parsing join room schema");
                tracing::debug!("received join room: {:#?}", room);
                self.leave_room().await;

                let room = self.global.get_room(room.id.into());
                if let Some(room) = room {
                    tracing::debug!("found room: {:#?}", room);
                    match room.subscribe() {
                        Some(mut subscription) => {
                            tracing::debug!(
                                "subscribed to room successfully, sending system message"
                            );
                            _ = subscription.send_hello(&self.persona).await;

                            self.room_subscription = Some(subscription)
                        }
                        None => {
                            tracing::debug!("room is full, cannot join")
                        }
                    }
                }
            }
            Opcode::SendMessage => {
                let msg: responses::SendMessage =
                    serde_json::from_value(data.data).expect("failed parsing send message schema");

                tracing::debug!("received send message: {:#?}", msg);

                let Some(ref room) = self.room_subscription else {
                    tracing::debug!("no room joined, ignoring request");
                    return;
                };

                let _ = room.send(RoomMessage {
                    persona: MessagePersona::from_persona(self.id, &self.persona),
                    message: msg.message,
                });
            }
            Opcode::CreateRoom => {
                tracing::debug!("received create room request");
                self.leave_room().await;

                let room = self.global.insert_room(Room::new_private());
                tracing::debug!("created and joining new private room with id {:?}", room.id);

                match room.subscribe() {
                    Some(mut subscription) => {
                        tracing::debug!("subscribed to room successfully, sending system message");
                        _ = subscription.send_hello(&self.persona).await;

                        self.room_subscription = Some(subscription)
                    }
                    None => {
                        tracing::debug!("room is full, cannot join")
                    }
                }
            }
            _ => (),
        }
    }

    async fn leave_room(&mut self) {
        if let Some(mut room) = self.room_subscription.take() {
            _ = room.send_bye(&self.persona).await;
            tracing::debug!("socket {} is leaving room {}", self.id, room.room.id.id());
        }
    }
}

#[axum::debug_handler]
pub async fn handler(
    State(global): State<Arc<GlobalState>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |ws| async move {
        tracing::debug!("accepting new socket connection");
        let guard = global.active_connection_guard();

        let mut socket = SocketConnection::new(ws, guard, global);
        tokio::spawn(async move { socket.serve().await });
    })
}

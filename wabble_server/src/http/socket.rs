use std::{sync::Arc, time::Duration};

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{self, Message, WebSocket},
    },
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    global::{ActiveConnectionGuard, GlobalState},
    responses::{self, SocketResponse, TSocketResponse},
    room::Persona,
};

#[derive(Debug)]
struct SocketConnection {
    id: Uuid,
    heartbeat: tokio::time::Interval,
    socket: WebSocket,
    persona: Persona,
    global: Arc<GlobalState>,
    _guard: ActiveConnectionGuard,
}

impl SocketConnection {
    fn new(socket: WebSocket, guard: ActiveConnectionGuard, global: Arc<GlobalState>) -> Self {
        Self {
            id: Uuid::new_v4(),
            heartbeat: tokio::time::interval(Duration::from_secs(30)),
            socket,
            persona: Persona::default(),
            global,
            _guard: guard,
        }
    }

    async fn send(&mut self, data: impl TSocketResponse + serde::Serialize) {
        // let data = serde_json::to_string(&data).expect("tragically failed to serialize data");

        self.socket
            .send(SocketResponse::new(data).into())
            .await
            .expect("failed to send message to socket");
    }

    async fn serve(&mut self) {
        println!("{:#?}", self);
        self.send(responses::Handshake {
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
                            tracing::debug!("received message: {}", s);
                        }
                        Some(Ok(_)) => {}
                        Some(Err(e)) => tracing::debug!("client disconnected abruptly: {e}"),
                        None => break,
                    }
                }
            }
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
        // loop {
        //     tokio::select! {
        //         // Since `ws` is a `Stream`, it is by nature cancel-safe.
        //         res = ws.recv() => {
        //             match res {
        //                 Some(Ok(ws::Message::Text(s))) => {
        //                     tracing::debug!("received message: {}", s);
        //                 }
        //                 Some(Ok(_)) => {}
        //                 Some(Err(e)) => tracing::debug!("client disconnected abruptly: {e}"),
        //                 None => break,
        //             }
        //         }
        //     }
        // }
    })
}

use std::{
    net::{Ipv4Addr, SocketAddrV4},
    sync::Arc,
};

use axum::{
    Router,
    routing::{any, get},
};
use tokio::{net::TcpSocket, sync::oneshot};

use crate::global::GlobalState;

mod socket;

fn routes(global: &Arc<GlobalState>) -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/socket", any(socket::handler))
        .with_state(global.clone())
}

pub async fn run(
    global: Arc<GlobalState>,
    shutdown_signal: oneshot::Receiver<()>,
) -> anyhow::Result<()> {
    tracing::info!("Listening on http://127.0.0.1:8080");

    let socket = TcpSocket::new_v4()?;

    socket.set_reuseaddr(true)?;
    socket.set_nodelay(true)?;

    socket.bind(std::net::SocketAddr::V4(SocketAddrV4::new(
        Ipv4Addr::new(127, 0, 0, 1),
        8080,
    )))?;
    let listener = socket.listen(1024)?;

    let routes = routes(&global);
    axum::serve(listener, routes)
        .with_graceful_shutdown(async move { _ = shutdown_signal.await })
        .await
        .expect("Failed to start the HTTP server");

    Ok(())
}

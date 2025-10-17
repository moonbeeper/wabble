use std::net::{Ipv4Addr, SocketAddrV4};

use axum::{Router, routing::get};
use tokio::{net::TcpSocket, sync::oneshot};

fn routes() -> Router {
    Router::new().route("/", get(|| async { "Hello, World!" }))
}

pub async fn run(shutdown_signal: oneshot::Receiver<()>) -> anyhow::Result<()> {
    tracing::info!("Listening on http://127.0.0.1:8080");

    let socket = TcpSocket::new_v4()?;

    socket.set_reuseaddr(true)?;
    socket.set_nodelay(true)?;

    socket.bind(std::net::SocketAddr::V4(SocketAddrV4::new(
        Ipv4Addr::new(127, 0, 0, 1),
        8080,
    )))?;
    let listener = socket.listen(1024)?;

    axum::serve(listener, routes())
        .with_graceful_shutdown(async move { _ = shutdown_signal.await })
        .await
        .expect("Failed to start the HTTP server");

    Ok(())
}

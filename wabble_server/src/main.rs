use std::{sync::Arc, time::Duration};

use rand::Rng;
use tokio::sync::oneshot;
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

pub mod global;
mod http;
pub mod responses;
pub mod room;

const FACES: &[&str] = &[":)", ":D", ":P", ":3"]; // astetic facses

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let face = FACES[rand::rng().random_range(0..FACES.len())];
    tracing::info!("heelo world from wabble server {}", face);

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let global = Arc::new(global::GlobalState::new());

    let http_server = tokio::spawn(http::run(global, shutdown_rx));

    let shutdown = tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        tracing::info!("Received ctrl-c signal, shutting down...");
        shutdown_tx.send(()).ok();

        tokio::time::timeout(Duration::from_secs(60), tokio::signal::ctrl_c())
            .await
            .ok();
    });

    tokio::select! {
        r = http_server => {
            match r {
                Ok(_) => tracing::info!("HTTP server exited successfully"),
                Err(e) => tracing::error!("HTTP server exited with error: {:?}", e),
            }
        }
        _ = shutdown => {
            tracing::info!("Force shutdown..");
        }
    }
}

use std::time::Duration;

use tokio::sync::oneshot;

mod http;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    tracing::info!("heelo world");

    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let http_server = tokio::spawn(http::run(shutdown_rx));

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

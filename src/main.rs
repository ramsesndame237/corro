mod api;
mod error;
mod settings;
mod storage;

use std::sync::Arc;

use anyhow::Context;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use settings::{LogFormat, Settings};
use storage::NullBackend;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = Settings::load().context("Failed to load configuration")?;

    init_tracing(&cfg);

    info!(
        version = env!("CARGO_PKG_VERSION"),
        host = %cfg.server.host,
        port = cfg.server.port,
        storage_path = %cfg.storage.path.display(),
        "Starting Corro"
    );

    // Issue #7 will replace NullBackend with FilesystemBackend
    let backend = Arc::new(NullBackend);

    let app = api::build_router(backend, cfg.server.request_timeout_secs);

    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("Failed to bind to {addr}"))?;

    info!(addr = %addr, "Server listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Server error")?;

    info!("Server stopped cleanly");
    Ok(())
}

fn init_tracing(cfg: &Settings) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&cfg.log.level));

    match cfg.log.format {
        LogFormat::Json => {
            tracing_subscriber::registry()
                .with(filter)
                .with(fmt::layer().json())
                .init();
        }
        LogFormat::Pretty => {
            tracing_subscriber::registry()
                .with(filter)
                .with(fmt::layer().pretty())
                .init();
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, draining in-flight requests...");
}

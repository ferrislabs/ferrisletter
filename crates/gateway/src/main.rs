use clap::Parser;
use dotenvy::dotenv;
use gateway::{AppState, GatewayConfig, build_router};
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = GatewayConfig::parse();
    let listen_addr = config.listen_addr;

    info!(
        oidc_provider = ?config.oidc_provider,
        oidc_issuer = %config.oidc_expected_issuer,
        mcp_server_url = %config.mcp_server_url,
        "gateway starting"
    );

    let state = AppState::new(config)?;
    let app = build_router(state);

    let listener = TcpListener::bind(listen_addr).await?;
    info!(addr = %listen_addr, "gateway listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
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

    info!("shutdown signal received");
}

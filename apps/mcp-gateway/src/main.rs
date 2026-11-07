use clap::Parser;
use mcp_gateway::{app::router, build_state, config::Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::parse();
    telemetry::init(&config.log, "mestier-mcp-gateway", &config.observability)?;
    let app = router(build_state(&config)?);
    let listener = tokio::net::TcpListener::bind(config.listen).await?;
    tracing::info!(listen = %config.listen, "mcp gateway started");
    axum::serve(listener, app).await?;
    Ok(())
}

use std::net::SocketAddr;

use args::{log::LogArgs, observability::ObservabilityArgs};
use clap::Parser;
use mcp_server::{ApiClient, FilePolicy, router_with};

#[derive(Parser)]
#[command(about = "Mestier MCP server: private, reached only through the gateway")]
struct Args {
    #[arg(long, env = "MCP_SERVER_LISTEN", default_value = "0.0.0.0:8081")]
    listen: SocketAddr,
    #[arg(
        long,
        env = "MCP_SERVER_API_URL",
        default_value = "http://localhost:3456"
    )]
    api_url: String,
    #[arg(
        long,
        env = "MCP_SERVER_FILE_HOSTS",
        value_delimiter = ',',
        default_value = "oaiusercontent.com,openai.com,chatgpt.com"
    )]
    file_hosts: Vec<String>,
    #[command(flatten)]
    log: LogArgs,
    #[command(flatten)]
    observability: ObservabilityArgs,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    telemetry::init(&args.log, "mestier-mcp-server", &args.observability)?;
    let listener = tokio::net::TcpListener::bind(args.listen).await?;
    tracing::info!(listen = %args.listen, api = %args.api_url, "mcp-server started");
    axum::serve(
        listener,
        router_with(
            ApiClient::new(args.api_url),
            FilePolicy::new(args.file_hosts),
        ),
    )
    .await?;
    Ok(())
}

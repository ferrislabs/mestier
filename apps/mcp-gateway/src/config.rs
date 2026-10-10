use std::{net::SocketAddr, time::Duration};

use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ClientAuthMethod {
    Post,
    Basic,
}

#[derive(Debug, Clone, Parser)]
#[command(name = "mcp-gateway")]
pub struct Config {
    #[arg(long, env = "MCP_GATEWAY_LISTEN", default_value = "0.0.0.0:8080")]
    pub listen: SocketAddr,

    #[arg(long, env = "MCP_GATEWAY_RESOURCE_URL")]
    pub resource_url: String,

    #[arg(long, env = "MCP_GATEWAY_ISSUER")]
    pub issuer: String,

    #[arg(long, env = "MCP_GATEWAY_JWKS_REFRESH_SECS", default_value_t = 300)]
    pub jwks_refresh_secs: u64,

    #[arg(long, env = "MCP_GATEWAY_PDP_URL")]
    pub pdp_url: String,

    #[arg(long, env = "MCP_GATEWAY_EXCHANGE_CLIENT_ID")]
    pub exchange_client_id: String,

    #[arg(
        long,
        env = "MCP_GATEWAY_EXCHANGE_CLIENT_SECRET",
        hide_env_values = true
    )]
    pub exchange_client_secret: String,

    #[arg(
        long,
        env = "MCP_GATEWAY_EXCHANGE_AUDIENCE",
        default_value = "mestier-api"
    )]
    pub exchange_audience: String,

    #[arg(
        long,
        env = "MCP_GATEWAY_EXCHANGE_AUTH_METHOD",
        value_enum,
        default_value = "post"
    )]
    pub exchange_auth_method: ClientAuthMethod,

    #[arg(long, env = "MCP_GATEWAY_UPSTREAM_URL")]
    pub upstream_url: String,
}

impl Config {
    pub fn jwks_refresh_interval(&self) -> Duration {
        Duration::from_secs(self.jwks_refresh_secs)
    }
}

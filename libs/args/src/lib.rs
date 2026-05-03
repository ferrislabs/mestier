use clap::Parser;
use common::Config;

use crate::{
    auth::AuthArgs, database::DatabaseArgs, log::LogArgs, observability::ObservabilityArgs,
    rate_limit::RateLimitArgs, server::ServerArgs,
};

pub mod auth;
pub mod database;
pub mod log;
pub mod observability;
pub mod rate_limit;
pub mod server;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[command(flatten)]
    pub log: LogArgs,

    #[command(flatten)]
    pub db: DatabaseArgs,

    #[command(flatten)]
    pub auth: AuthArgs,

    #[command(flatten)]
    pub server: ServerArgs,

    #[command(flatten)]
    pub observability: ObservabilityArgs,

    #[command(flatten)]
    pub rate_limit: RateLimitArgs,
}

impl From<Args> for Config {
    fn from(value: Args) -> Self {
        Self {
            auth: value.auth.into(),
            database: value.db.into(),
            rate_limit: value.rate_limit.into(),
        }
    }
}

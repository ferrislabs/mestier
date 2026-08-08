use clap::Parser;
use common::Config;

use crate::{
    auth::AuthArgs, automation::AutomationArgs, database::DatabaseArgs,
    file_storage::FileStorageArgs, log::LogArgs, observability::ObservabilityArgs,
    rate_limit::RateLimitArgs, server::ServerArgs,
};

pub mod auth;
pub mod automation;
pub mod database;
pub mod file_storage;
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

    #[command(flatten)]
    pub file_storage: FileStorageArgs,

    #[command(flatten)]
    pub automation: AutomationArgs,
}

impl From<Args> for Config {
    fn from(value: Args) -> Self {
        Self {
            auth: value.auth.into(),
            automation: value.automation.into(),
            database: value.db.into(),
            rate_limit: value.rate_limit.into(),
            file_storage: value.file_storage.into(),
        }
    }
}

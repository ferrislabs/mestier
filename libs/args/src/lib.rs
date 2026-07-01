use clap::Parser;
use common::Config;

use crate::{
    auth::AuthArgs, database::DatabaseArgs, file_storage::FileStorageArgs, log::LogArgs,
    observability::ObservabilityArgs, rate_limit::RateLimitArgs, server::ServerArgs,
};

pub mod auth;
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

    /// Secret used to sign document access tokens (HMAC-SHA256).
    /// Falls back to `--auth-client-secret` / `AUTH_CLIENT_SECRET` when absent.
    #[arg(
        long = "document-signing-secret",
        env = "DOCUMENT_SIGNING_SECRET",
        name = "DOCUMENT_SIGNING_SECRET",
        long_help = "Secret for HMAC-SHA256 document token signing; defaults to AUTH_CLIENT_SECRET"
    )]
    pub document_signing_secret: Option<String>,
}

impl From<Args> for Config {
    fn from(value: Args) -> Self {
        // Fall back to the auth client secret when DOCUMENT_SIGNING_SECRET is not set.
        let document_signing_secret = value
            .document_signing_secret
            .unwrap_or_else(|| value.auth.client_secret.clone());
        Self {
            auth: value.auth.into(),
            database: value.db.into(),
            rate_limit: value.rate_limit.into(),
            file_storage: value.file_storage.into(),
            document_signing_secret,
        }
    }
}

use std::sync::{LazyLock, Mutex};

use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::{ContextV7, Timestamp, Uuid};

#[derive(Clone, Debug)]
pub struct Config {
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub rate_limit: RateLimitConfig,
    pub file_storage: FileStorageConfig,
}

#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    pub redis_url: String,
    pub per_minute: u32,
}

#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct AuthConfig {
    pub issuer: String,
}

#[derive(Clone, Debug)]
pub struct FileStorageConfig {
    pub bucket: String,
    pub endpoint: String,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub force_path_style: bool,
    pub auto_create_bucket: bool,
    pub key_prefix: String,
    pub max_upload_bytes: u64,
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("resource not found")]
    NotFound,

    #[error("conflict: {0}")]
    Conflict(String),

    /// The actor was authenticated but the policy engine refused the
    /// action. `reason` carries an optional, human-readable explanation
    /// for logs and (when safe) for the API response.
    #[error("forbidden{}", .reason.as_ref().map(|r| format!(": {r}")).unwrap_or_default())]
    Forbidden { reason: Option<String> },

    #[error("database error: {0}")]
    Database(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// Shared monotonic counter so that uuid v7 ids minted within the same
/// millisecond stay strictly ordered (used for read-state unread detection and
/// message cursor pagination, which rely on `id` order matching creation order).
/// `ContextV7` is not `Sync` (it holds `Cell`s), so it is guarded by a `Mutex`;
/// the lock is held only for the counter bump, so contention is negligible.
static UUID_CONTEXT: LazyLock<Mutex<ContextV7>> = LazyLock::new(|| Mutex::new(ContextV7::new()));

pub fn generate_timestamp() -> (DateTime<Utc>, Timestamp) {
    let now = Utc::now();
    let seconds = now.timestamp().try_into().unwrap_or(0);

    let context = UUID_CONTEXT.lock().expect("uuid v7 context mutex poisoned");
    let timestamp = Timestamp::from_unix(&*context, seconds, now.timestamp_subsec_nanos());

    (now, timestamp)
}

pub fn generate_uuid_v7() -> Uuid {
    let (_, timestamp) = generate_timestamp();
    Uuid::new_v7(timestamp)
}

use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct UserId(pub Uuid);

impl FromStr for UserId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(UserId)
    }
}

impl Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct OrganizationId(pub Uuid);

impl FromStr for OrganizationId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(OrganizationId)
    }
}

impl Display for OrganizationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct RoleId(pub Uuid);

impl FromStr for RoleId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(RoleId)
    }
}

impl Display for RoleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_uuid_v7_is_strictly_monotonic() {
        // Ids minted back-to-back must be strictly increasing so that `id > marker`
        // reliably means "newer" (read-state unread detection, message cursor pagination).
        let mut prev = generate_uuid_v7();
        for _ in 0..10_000 {
            let next = generate_uuid_v7();
            assert!(
                next > prev,
                "uuid v7 must be strictly increasing, got {prev} then {next}"
            );
            prev = next;
        }
    }

    #[test]
    fn generate_uuid_v7_has_version_7() {
        assert_eq!(generate_uuid_v7().get_version_num(), 7);
    }
}

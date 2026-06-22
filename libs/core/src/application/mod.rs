use std::sync::Arc;

use auth::{AuthService, FerrisKeyRepository};
use authz::LocalPolicyEngine;
use common::{Config, CoreError};
use rate_limit::{Quota, RateLimitService, RedisRateLimiter};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::domain::file_storage::service::FileStorageService;
use crate::domain::role::Permissions;
use crate::infrastructure::file_storage::S3FileStorage;
use crate::infrastructure::postgres::error::map_sqlx_error;
use crate::infrastructure::realtime::{EventHub, RealtimeEventPublisher};

pub mod customer;
pub mod customer_contact;
pub mod customer_context;
pub mod discord_category;
pub mod discord_channel;
pub mod discord_message;
pub mod discord_presence;
pub mod discord_webhook;
pub mod employee;
pub mod equipment;
pub mod member;
pub mod organization;
pub mod policy;
pub mod product;
pub mod quote;
pub mod role;
pub mod service_rate;
pub mod user;

pub type MestierAuthService = AuthService<FerrisKeyRepository>;
pub type MestierFileStorageService = FileStorageService<S3FileStorage>;
pub type MestierRateLimitService = RateLimitService<RedisRateLimiter>;

/// In-process Policy Decision Point used by Mestier's services. Aliased so
/// callers can swap the concrete engine later (e.g. for a remote PDP)
/// with a single type change.
pub type MestierAuthorizer = LocalPolicyEngine;

/// Builds the default action → required permission bits map. The bit
/// values come from [`Permissions`] so the service-side bitfield stays
/// the single source of truth.
pub fn default_authorizer() -> MestierAuthorizer {
    LocalPolicyEngine::builder()
        .action("organization.update", Permissions::MANAGE_ORG.0)
        .action("organization.delete", Permissions::MANAGE_ORG.0)
        .action("member.invite", Permissions::MANAGE_MEMBERS.0)
        .action("member.remove", Permissions::MANAGE_MEMBERS.0)
        .action("role.assign", Permissions::MANAGE_ROLES.0)
        .action("role.manage", Permissions::MANAGE_ROLES.0)
        .action("category.manage", Permissions::MANAGE_CHANNELS.0)
        .action("channel.manage", Permissions::MANAGE_CHANNELS.0)
        .action("message.delete_any", Permissions::MANAGE_CHANNELS.0)
        .action("webhook.manage", Permissions::MANAGE_WEBHOOKS.0)
        .build()
}

#[derive(Clone)]
pub struct MestierUseCase {
    pub(crate) pool: PgPool,
    pub(crate) authz: MestierAuthorizer,
    pub(crate) events: Arc<RealtimeEventPublisher>,
}

impl MestierUseCase {
    pub fn new(
        pool: PgPool,
        authz: MestierAuthorizer,
        events: Arc<RealtimeEventPublisher>,
    ) -> Self {
        Self {
            pool,
            authz,
            events,
        }
    }
}

#[derive(Clone)]
pub struct MestierService {
    pub auth: MestierAuthService,
    pub file_storage: MestierFileStorageService,
    pub usecase: MestierUseCase,
    pub rate_limit: MestierRateLimitService,
    pub rate_limit_quota: Quota,
    /// Subscribe-side handle for the in-process event bus.  Plan 3 (WS gateway)
    /// reads this to subscribe to per-org broadcast channels.
    pub events: EventHub,
}

impl MestierService {
    pub fn new(
        auth: MestierAuthService,
        file_storage: MestierFileStorageService,
        usecase: MestierUseCase,
        rate_limit: MestierRateLimitService,
        rate_limit_quota: Quota,
        events: EventHub,
    ) -> Self {
        Self {
            auth,
            file_storage,
            usecase,
            rate_limit,
            rate_limit_quota,
            events,
        }
    }
}

pub async fn create_service(config: Config) -> Result<MestierService, CoreError> {
    let auth_repo = FerrisKeyRepository::new(config.auth.issuer, None);
    let auth = AuthService::new(auth_repo);
    let s3_storage = S3FileStorage::from_config(&config.file_storage);
    if config.file_storage.auto_create_bucket {
        s3_storage.ensure_bucket().await?;
    }
    let file_storage = FileStorageService::new(s3_storage, config.file_storage.key_prefix);

    let db_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        config.database.username,
        config.database.password,
        config.database.host,
        config.database.port,
        config.database.name,
    );
    let pool = PgPoolOptions::new()
        .connect(&db_url)
        .await
        .map_err(map_sqlx_error)?;

    let limiter = RedisRateLimiter::connect(&config.rate_limit.redis_url)
        .await
        .map_err(|e| CoreError::Internal(format!("redis connection failed: {e}")))?;
    let rate_limit = RateLimitService::new(limiter);
    let rate_limit_quota = Quota::per_minute(config.rate_limit.per_minute);

    let hub = EventHub::new();
    let publisher = Arc::new(RealtimeEventPublisher::new(hub.clone()));

    Ok(MestierService::new(
        auth,
        file_storage,
        MestierUseCase::new(pool, default_authorizer(), publisher),
        rate_limit,
        rate_limit_quota,
        hub,
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn mestier_use_case_has_events_field() {
        // Verifies the struct compiles with the new field.
        // Actual behavior is tested in Task 10.
        let _ = std::any::type_name::<crate::application::MestierUseCase>();
    }
}

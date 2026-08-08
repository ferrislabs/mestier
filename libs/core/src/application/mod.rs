use auth::{AuthService, FerrisKeyRepository};
use authz::LocalPolicyEngine;
use common::{AutomationConfig, Config, CoreError};
use rate_limit::{Quota, RateLimitService, RedisRateLimiter};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::domain::file_storage::service::FileStorageService;
use crate::domain::role::Permissions;
use crate::infrastructure::automation::webhook::{
    WebhookDeliveryHandler, address_policy::PrivateNetworkAccess, secret::SecretCipher,
};
use crate::infrastructure::automation::worker::{WorkerSchedule, run_automation_worker};
use crate::infrastructure::file_storage::S3FileStorage;
use crate::infrastructure::postgres::error::map_sqlx_error;
use crate::infrastructure::realtime::EventHub;
use common::UserId;
use events::Actor;

pub mod absence;
pub mod authorization;
pub mod automation;
pub mod customer;
pub mod customer_contact;
pub mod customer_context;
pub mod discord_category;
pub mod discord_channel;
pub mod discord_message;
pub mod discord_notification;
pub mod discord_overwrite;
pub mod discord_presence;
pub mod discord_read_state;
pub mod discord_webhook;
pub mod employee;
pub mod equipment;
pub mod member;
pub mod organization;
pub mod planning;
pub mod policy;
pub mod product;
pub mod quote;
pub mod role;
pub mod service_rate;
pub mod task;
pub mod task_comment;
pub mod task_label;
pub mod user;
pub mod work_time;

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
    pub(crate) hub: EventHub,
    /// Who the events produced through this handle are attributed to.
    /// `System` unless a handler asked for an acting view via [`Self::acting_as`].
    pub(crate) actor: Actor,
    /// Absent when the instance has no `AUTOMATION_SECRET_KEY`. Typed rather
    /// than documented: without it there is no way to seal a webhook secret,
    /// so endpoint creation cannot even be attempted.
    pub(crate) secret_cipher: Option<std::sync::Arc<SecretCipher>>,
}

impl MestierUseCase {
    pub fn new(pool: PgPool, authz: MestierAuthorizer, hub: EventHub) -> Self {
        Self {
            pool,
            authz,
            hub,
            actor: Actor::System,
            secret_cipher: None,
        }
    }

    pub fn with_secret_cipher(mut self, cipher: std::sync::Arc<SecretCipher>) -> Self {
        self.secret_cipher = Some(cipher);
        self
    }

    /// The cipher, or a refusal an operator can act on.
    pub(crate) fn cipher(&self) -> Result<&SecretCipher, CoreError> {
        self.secret_cipher.as_deref().ok_or_else(|| {
            CoreError::Conflict(
                "this instance has no automation secret key configured, so webhook secrets \
                 cannot be stored"
                    .to_owned(),
            )
        })
    }

    /// A view of this use case whose events are attributed to `user_id`.
    ///
    /// Takes the **local** `users.id`, not the FerrisKey subject: the subject
    /// is an opaque string, and an event's actor has to be joinable against
    /// the users table. Handlers get the local id from `require_org_membership`,
    /// which already resolves it to check the caller belongs to the
    /// organization — so an actor cannot be obtained without that check having
    /// passed.
    ///
    /// Cloning is a handful of refcount bumps: the pool, the policy engine and
    /// the hub are all `Arc`-backed.
    pub fn acting_as(&self, user_id: UserId) -> Self {
        Self {
            actor: Actor::user(user_id.0),
            ..self.clone()
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
    let cipher = config
        .automation
        .secret_key
        .as_deref()
        .map(SecretCipher::from_base64)
        .transpose()?
        .map(std::sync::Arc::new);

    let mut usecase = MestierUseCase::new(pool.clone(), default_authorizer(), hub.clone());
    if let Some(cipher) = cipher.clone() {
        usecase = usecase.with_secret_cipher(cipher);
    }

    spawn_automation_worker(&config.automation, pool, usecase.clone(), cipher)?;

    Ok(MestierService::new(
        auth,
        file_storage,
        usecase,
        rate_limit,
        rate_limit_quota,
        hub,
    ))
}

/// Starts the background loop that fans events out and delivers them.
///
/// Silently doing nothing when no key is configured would be the wrong
/// failure: an operator would see events pile up with no explanation. It logs
/// loudly instead, and the API refuses to create an endpoint for the same
/// reason — no key means no way to store a secret.
fn spawn_automation_worker(
    config: &AutomationConfig,
    pool: PgPool,
    usecase: MestierUseCase,
    cipher: Option<std::sync::Arc<SecretCipher>>,
) -> Result<(), CoreError> {
    let Some(cipher) = cipher else {
        tracing::warn!(
            "no AUTOMATION_SECRET_KEY configured: the automation worker will not start and \
             webhook endpoints cannot be created"
        );
        return Ok(());
    };

    let access = if config.allow_private_network {
        tracing::warn!(
            "webhooks may reach private addresses. Correct for a single-tenant instance; \
             on a shared one a tenant can borrow this server's network rights"
        );
        PrivateNetworkAccess::Allowed
    } else {
        PrivateNetworkAccess::Denied
    };

    let handler = WebhookDeliveryHandler::new(pool, cipher, access)?;
    // Named after the host so a delivery stuck `in_flight` points at the
    // machine that was holding it.
    let worker = hostname().unwrap_or_else(|| "mestier".to_owned());

    tokio::spawn(run_automation_worker(
        usecase,
        handler,
        worker,
        WorkerSchedule::default(),
    ));

    Ok(())
}

fn hostname() -> Option<String> {
    std::env::var("HOSTNAME")
        .ok()
        .filter(|name| !name.is_empty())
}

// `MestierUseCase` deliberately holds no event publisher. It is long-lived and
// cloned into every request, so any publisher stored here would be one shared
// buffer for the whole process — which is precisely how events reached the
// wrong organization. `#[transactional(events)]` builds one per transaction
// instead; see `mestier_macros::transactional`.

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    /// `connect_lazy` builds a pool without touching the network, so the actor
    /// logic can be tested without a database it does not use.
    fn use_case() -> MestierUseCase {
        let pool = PgPool::connect_lazy("postgres://unused:unused@localhost/unused")
            .expect("a lazy pool needs no server");
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    #[tokio::test]
    async fn a_use_case_acts_as_the_system_until_told_otherwise() {
        assert_eq!(use_case().actor, Actor::System);
    }

    #[tokio::test]
    async fn acting_as_a_user_attributes_events_to_that_user() {
        let user_id = UserId(Uuid::from_u128(1));

        let acting = use_case().acting_as(user_id);

        assert_eq!(acting.actor, Actor::user(user_id.0));
    }

    /// The handler holds a shared `AppState`; asking for an acting view must
    /// not change what every other request is attributed to.
    #[tokio::test]
    async fn acting_as_leaves_the_original_untouched() {
        let base = use_case();

        let _acting = base.acting_as(UserId(Uuid::from_u128(1)));

        assert_eq!(base.actor, Actor::System);
    }
}

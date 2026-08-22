use std::sync::Arc;

use auth::{AuthService, FerrisKeyRepository};
use authz::LocalPolicyEngine;
use common::{AutomationConfig, Config, CoreError};
use rate_limit::{Quota, RateLimitService, RedisRateLimiter};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::domain::file_storage::service::FileStorageService;
use crate::domain::role::Permissions;
use crate::infrastructure::automation::retention::{RetentionWorkerSchedule, run_retention_worker};
use crate::infrastructure::automation::webhook::{
    address_policy::PrivateNetworkAccess, secret::SecretCipher,
};
use crate::infrastructure::automation::worker::{WorkerSchedule, run_automation_worker};
use crate::infrastructure::file_storage::S3FileStorage;
use crate::infrastructure::postgres::error::map_sqlx_error;
use crate::infrastructure::realtime::EventHub;
use common::UserId;
use events::Actor;
use uuid::Uuid;

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
pub mod invitation;
pub mod member;
pub mod organization;
pub mod planning;
pub mod policy;
pub mod product;
pub mod profitability;
pub mod project;
pub mod quote;
pub mod role;
pub mod service_rate;
pub mod task;
pub mod task_comment;
pub mod task_label;
#[cfg(test)]
pub(crate) mod test_support;
pub mod time_entry;
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
        .action("member.manage", Permissions::MANAGE_MEMBERS.0)
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
    /// Process-wide key sealing credential data at rest. `None` when the
    /// instance has no `AUTOMATION_SECRET_KEY`: it still boots, and every
    /// credential operation that needs to seal something refuses loudly
    /// instead of silently storing plaintext — the same choice the
    /// automation worker makes at startup.
    pub(crate) cipher: Option<Arc<SecretCipher>>,
}

impl MestierUseCase {
    pub fn new(pool: PgPool, authz: MestierAuthorizer, hub: EventHub) -> Self {
        Self {
            pool,
            authz,
            hub,
            actor: Actor::System,
            cipher: None,
        }
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

    /// A view of this use case whose events are attributed to the run
    /// `run_id` — strictly the shape of [`Self::acting_as`], substituting the
    /// run engine's identity for a human's.
    ///
    /// The actor lives on the value, not on a call-stack-scoped context, so
    /// whatever this handle calls next — at any depth, through any number of
    /// further use-case methods — emits with this actor: `#[transactional]`
    /// always builds its emitter from `self.actor`, never from how deep the
    /// call chain was to get there.
    pub fn acting_as_automation(&self, run_id: Uuid) -> Self {
        Self {
            actor: Actor::automation(run_id),
            ..self.clone()
        }
    }

    /// Attaches the key that seals credential data at rest.
    ///
    /// A method rather than a public field so the composition root is the only
    /// place that can set it, and so it reads the same way as [`Self::acting_as`]
    /// — this type is configured by returning a new view of itself, never by
    /// mutating one from another module.
    pub fn with_cipher(mut self, cipher: Option<Arc<SecretCipher>>) -> Self {
        self.cipher = cipher;
        self
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
    let cipher = match config.automation.secret_key.as_deref() {
        Some(encoded) => Some(Arc::new(SecretCipher::from_base64(encoded)?)),
        None => None,
    };
    let usecase =
        MestierUseCase::new(pool.clone(), default_authorizer(), hub.clone()).with_cipher(cipher);

    spawn_automation_worker(usecase.clone(), &config.automation);

    Ok(MestierService::new(
        auth,
        file_storage,
        usecase,
        rate_limit,
        rate_limit_quota,
        hub,
    ))
}

/// Starts the background loop that fans events out into runs and executes
/// them.
///
/// The worker always starts, unlike the webhook delivery pass it replaces:
/// a graph made only of `flow.*` and `mestier.*` connectors runs perfectly
/// well without a key. But a connector carrying a credential has to open it,
/// so without `AUTOMATION_SECRET_KEY` those steps fail one by one instead of
/// the whole loop refusing to start. That degradation is visible in the run
/// inspector rather than silent, and it is announced here so an operator does
/// not have to discover it one failed run at a time.
fn spawn_automation_worker(usecase: MestierUseCase, config: &AutomationConfig) {
    // Named after the host so a run stuck `running` points at the machine
    // that was holding it.
    let worker = hostname().unwrap_or_else(|| "mestier".to_owned());

    if usecase.cipher.is_none() {
        tracing::warn!(
            "no AUTOMATION_SECRET_KEY configured: workflows run, but any connector using a \
             credential will fail because its secret cannot be opened"
        );
    }

    let private_network = if config.allow_private_network {
        tracing::warn!(
            "workflow connectors may reach private addresses. Correct for a single-tenant \
             instance; on a shared one a tenant can borrow this server's network rights"
        );
        PrivateNetworkAccess::Allowed
    } else {
        PrivateNetworkAccess::Denied
    };

    tokio::spawn(run_automation_worker(
        usecase.clone(),
        worker,
        WorkerSchedule::default(),
        private_network,
    ));

    // Its own loop, deliberately far slower than the engine's: retention is
    // measured in days, and a purge that ran as often as the worker would take
    // the same locks for nothing. Without it `automation.event` and
    // `automation.run` grow without bound — one 500-item loop is 500 steps.
    tokio::spawn(run_retention_worker(
        usecase,
        RetentionWorkerSchedule::default(),
    ));
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
    use common::{OrganizationId, generate_uuid_v7};
    use uuid::Uuid;

    use crate::application::test_support::{dev_pool, purge};

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

    #[tokio::test]
    async fn acting_as_automation_attributes_events_to_that_run() {
        let run_id = Uuid::from_u128(1);

        let acting = use_case().acting_as_automation(run_id);

        assert_eq!(acting.actor, Actor::automation(run_id));
    }

    /// Same discipline as [`acting_as_leaves_the_original_untouched`]: the
    /// run engine calls this once per connector it dispatches through, and
    /// must never mutate the shared handle every other caller holds.
    #[tokio::test]
    async fn acting_as_automation_leaves_the_original_untouched() {
        let base = use_case();

        let _acting = base.acting_as_automation(Uuid::from_u128(1));

        assert_eq!(base.actor, Actor::System);
    }

    async fn make_pool() -> PgPool {
        dev_pool().await
    }

    async fn seed_organization(pool: &PgPool) -> OrganizationId {
        let owner_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            owner_id,
            format!("owner-{owner_id}@example.com"),
            format!("owner-{owner_id}"),
            "Owner User",
            format!("sub-owner-{owner_id}"),
        )
        .execute(pool)
        .await
        .unwrap();

        let org_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organizations (id, name, slug, owner_id)
               VALUES ($1, $2, $3, $4)"#,
            org_id,
            "Test Org",
            format!("test-org-{org_id}"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();

        OrganizationId(org_id)
    }

    /// The acceptance criterion the whole mechanism exists for: an event
    /// emitted through `acting_as_automation` carries the run id — not just
    /// as a field on the Rust value (the two tests above), but all the way
    /// through `#[transactional]`'s emitter and into the committed row.
    ///
    /// `create_quote` is the vehicle because it is the module that already
    /// emits (`#[transactional(quote, emitter)]`); nothing about this test
    /// is specific to quotes. `#[transactional]` builds `__event_emitter`
    /// from `self.actor` inside its own expansion — a plain field read, not
    /// a value threaded explicitly through each intervening call — so
    /// whatever this handle calls, at whatever depth, emits with this actor.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn acting_as_automation_attributes_a_persisted_event_to_the_run() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool).await;
        let usecase = MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new());
        let customer = usecase
            .create_customer(crate::domain::customer::commands::CreateCustomerCommand {
                organization_id: org_id,
                status: crate::CustomerStatus::Prospect,
                pipeline_stage: crate::CustomerPipelineStage::New,
                name: "Automation depth customer".to_string(),
                phone: None,
                email: None,
            })
            .await
            .unwrap();
        let context = usecase
            .create_customer_context(
                crate::domain::customer_context::commands::CreateCustomerContextCommand {
                    customer_id: customer.id,
                    label: "Main site".to_string(),
                    address_line: None,
                    postal_code: None,
                    city: None,
                    photo_key: None,
                },
            )
            .await
            .unwrap();
        let run_id = generate_uuid_v7();

        let created = usecase
            .acting_as_automation(run_id)
            .create_quote(crate::domain::quote::commands::CreateQuoteCommand {
                organization_id: org_id,
                title: "Automated quote".to_string(),
                customer_id: customer.id,
                customer_context_id: context.id,
                lines: Vec::new(),
            })
            .await
            .unwrap();

        let row = sqlx::query!(
            r#"SELECT actor_kind, actor_id FROM automation.event
               WHERE name = 'quote.created' AND subject_id = $1"#,
            created.id.0,
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(row.actor_kind, "automation");
        assert_eq!(row.actor_id, Some(run_id));

        // This test had no cleanup at all, and left an organization, a user, a
        // customer, a context, a quote and an event behind on every run. Once the
        // automation suites moved to their own database it was the last fixture
        // still leaking into the shared one.
        let owner_id: Uuid =
            sqlx::query_scalar!("SELECT owner_id FROM organizations WHERE id = $1", org_id.0)
                .fetch_one(&pool)
                .await
                .unwrap();

        for statement in [
            "DELETE FROM automation.event WHERE org_id = $1",
            "DELETE FROM quote_lines WHERE org_id = $1",
            "DELETE FROM quotes WHERE org_id = $1",
            "DELETE FROM customer_contexts WHERE customer_id IN (SELECT id FROM customers WHERE org_id = $1)",
            "DELETE FROM customers WHERE org_id = $1",
            "DELETE FROM organizations WHERE id = $1",
        ] {
            purge(&pool, statement, org_id.0).await;
        }
        purge(&pool, "DELETE FROM users WHERE id = $1", owner_id).await;
    }
}

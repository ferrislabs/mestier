//! Brings up the automation API on a real socket, against a real database.
//!
//! Same shape as `handlers-invoice`'s own harness: `AppState` is built by
//! `handlers::state`, the same function the binary calls, and only the auth
//! issuer and the database are pointed elsewhere.
//!
//! #493's permission split needs four distinct callers, not the invoice
//! suite's two: full access (`token`), `VIEW_AUTOMATION` alone
//! (`view_only_token`), bare membership with no role at all
//! (`no_role_token`, mirroring `handlers-reporting`'s own harness), and a
//! member of a *second* organization who holds both bits there
//! (`outsider_token`) — the one that proves membership in the organization
//! actually being called is never bypassed by holding the bit somewhere
//! else.

use std::{net::SocketAddr, sync::Arc};

use args::Args;
use clap::Parser;
use mestier_core::Permissions;
use sqlx::PgPool;
use uuid::Uuid;

use crate::issuer;

pub struct App {
    pub base_url: String,
    /// Holds `VIEW_AUTOMATION | MANAGE_AUTOMATION` — the fixture's main
    /// caller, exercising full read/write access.
    pub token: String,
    /// Holds `VIEW_AUTOMATION` only.
    pub view_only_token: String,
    /// Membership, no role assignment at all — the bare "belongs to the
    /// organization" case #493's gate now refuses outright.
    pub no_role_token: String,
    /// A member of `other_organization_id`, not of `organization_id` at
    /// all, holding `VIEW_AUTOMATION | MANAGE_AUTOMATION` in that other
    /// organization.
    pub outsider_token: String,
    pub pool: PgPool,
    pub organization_id: Uuid,
    other_organization_id: Uuid,
    user_id: Uuid,
    view_only_user_id: Uuid,
    no_role_user_id: Uuid,
    outsider_user_id: Uuid,
}

/// Panics rather than skipping when the stack is down: the test is `#[ignore]`d,
/// so reaching this function is already a statement that the stack is up.
pub async fn start() -> App {
    let issuer_url = issuer::spawn();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set to run the http end-to-end tests");
    let redis_url = std::env::var("RATE_LIMIT_REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_owned());

    let pool = PgPool::connect(&database_url)
        .await
        .expect("connect to the test database");
    let fixture = seed(&pool).await;

    let args = Arc::new(Args::parse_from(args_for(
        &database_url,
        &redis_url,
        &issuer_url,
    )));
    let state = handlers::state(args)
        .await
        .expect("build AppState for the test");

    let router = handlers_automation::router(&state).with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind the test api");
    let addr = listener.local_addr().expect("read the test api address");
    // `ConnectInfo` the same way the binary supplies it: the rate-limit
    // middleware keys on the peer address and 500s without it.
    tokio::spawn(async move {
        let _ = axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await;
    });

    App {
        base_url: format!("http://{addr}"),
        token: issuer::mint(&fixture.sub),
        view_only_token: issuer::mint(&fixture.view_only_sub),
        no_role_token: issuer::mint(&fixture.no_role_sub),
        outsider_token: issuer::mint(&fixture.outsider_sub),
        pool,
        organization_id: fixture.organization_id,
        other_organization_id: fixture.other_organization_id,
        user_id: fixture.user_id,
        view_only_user_id: fixture.view_only_user_id,
        no_role_user_id: fixture.no_role_user_id,
        outsider_user_id: fixture.outsider_user_id,
    }
}

impl App {
    pub fn workflows_url(&self) -> String {
        format!(
            "{}/api/v1/organizations/{}/automation/workflows",
            self.base_url, self.organization_id
        )
    }

    pub fn workflow_trigger_url(&self, workflow_id: &str) -> String {
        format!("{}/{workflow_id}/trigger", self.workflows_url())
    }

    pub fn settings_url(&self) -> String {
        format!(
            "{}/api/v1/organizations/{}/automation/settings",
            self.base_url, self.organization_id
        )
    }

    pub fn evaluate_expression_url(&self) -> String {
        format!(
            "{}/api/v1/organizations/{}/automation/expressions/evaluate",
            self.base_url, self.organization_id
        )
    }

    /// Drops the fixture. Every table listed here references `organizations`
    /// without `ON DELETE CASCADE`, so one missing from the list makes the
    /// final delete fail — silently, since the errors are swallowed.
    pub async fn cleanup(&self) {
        for organization_id in [self.organization_id, self.other_organization_id] {
            for statement in [
                "DELETE FROM automation.settings WHERE org_id = $1",
                "DELETE FROM automation.event WHERE org_id = $1",
                "DELETE FROM organizations WHERE id = $1",
            ] {
                let _ = sqlx::query(statement)
                    .bind(organization_id)
                    .execute(&self.pool)
                    .await;
            }
        }

        for user_id in [
            self.user_id,
            self.view_only_user_id,
            self.no_role_user_id,
            self.outsider_user_id,
        ] {
            let _ = sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&self.pool)
                .await;
        }
    }
}

struct Fixture {
    sub: String,
    user_id: Uuid,
    view_only_sub: String,
    view_only_user_id: Uuid,
    no_role_sub: String,
    no_role_user_id: Uuid,
    outsider_sub: String,
    outsider_user_id: Uuid,
    organization_id: Uuid,
    other_organization_id: Uuid,
}

/// Two organizations: `organization_id`, the one every test calls into, with
/// three members (full access, view-only, bare membership); and
/// `other_organization_id`, whose only member holds both automation bits —
/// but there, not in `organization_id`.
///
/// Queries are unchecked on purpose: `query!` would demand a regenerated
/// `.sqlx` cache for statements that only ever run against a live database.
async fn seed(pool: &PgPool) -> Fixture {
    let organization_id = Uuid::now_v7();
    let (user_id, sub, member_id) = seed_person(pool, organization_id, true).await;

    let full_access_role_id = seed_role(
        pool,
        organization_id,
        "test-automation-manager",
        (Permissions::VIEW_AUTOMATION | Permissions::MANAGE_AUTOMATION).0,
    )
    .await;
    assign_role(pool, member_id, full_access_role_id).await;

    let view_only_role_id = seed_role(
        pool,
        organization_id,
        "test-automation-viewer",
        Permissions::VIEW_AUTOMATION.0,
    )
    .await;
    let (view_only_user_id, view_only_sub, view_only_member_id) =
        seed_person(pool, organization_id, false).await;
    assign_role(pool, view_only_member_id, view_only_role_id).await;

    let (no_role_user_id, no_role_sub, _) = seed_person(pool, organization_id, false).await;

    let other_organization_id = Uuid::now_v7();
    let (outsider_user_id, outsider_sub, outsider_member_id) =
        seed_person(pool, other_organization_id, true).await;
    let outsider_role_id = seed_role(
        pool,
        other_organization_id,
        "test-automation-manager",
        (Permissions::VIEW_AUTOMATION | Permissions::MANAGE_AUTOMATION).0,
    )
    .await;
    assign_role(pool, outsider_member_id, outsider_role_id).await;

    Fixture {
        sub,
        user_id,
        view_only_sub,
        view_only_user_id,
        no_role_sub,
        no_role_user_id,
        outsider_sub,
        outsider_user_id,
        organization_id,
        other_organization_id,
    }
}

/// A user and their seat in `organization_id`. `owns_organization` inserts
/// the organization row itself — only the first person seeded into a given
/// organization needs to do that.
async fn seed_person(
    pool: &PgPool,
    organization_id: Uuid,
    owns_organization: bool,
) -> (Uuid, String, Uuid) {
    let user_id = Uuid::now_v7();
    let sub = format!("sub-e2e-{user_id}");
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("artisan-{user_id}@example.com"))
    .bind(format!("artisan-{user_id}"))
    .bind("Artisan Test")
    .bind(&sub)
    .execute(pool)
    .await
    .expect("seed the user");

    if owns_organization {
        sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
            .bind(organization_id)
            .bind("Test Org")
            .bind(format!("test-org-{organization_id}"))
            .bind(user_id)
            .execute(pool)
            .await
            .expect("seed the organization");
    }

    let member_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name) VALUES ($1, $2, $3, $4)",
    )
    .bind(member_id)
    .bind(organization_id)
    .bind(user_id)
    .bind("Artisan Test")
    .execute(pool)
    .await
    .expect("seed the membership");

    (user_id, sub, member_id)
}

async fn seed_role(pool: &PgPool, organization_id: Uuid, name: &str, permissions: i64) -> Uuid {
    let role_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO roles (id, organization_id, name, permissions) VALUES ($1, $2, $3, $4)",
    )
    .bind(role_id)
    .bind(organization_id)
    .bind(name)
    .bind(permissions)
    .execute(pool)
    .await
    .expect("seed the role");

    role_id
}

async fn assign_role(pool: &PgPool, member_id: Uuid, role_id: Uuid) {
    sqlx::query("INSERT INTO member_roles (id, member_id, role_id) VALUES ($1, $2, $3)")
        .bind(Uuid::now_v7())
        .bind(member_id)
        .bind(role_id)
        .execute(pool)
        .await
        .expect("assign the role");
}

/// Only the three endpoints the test controls are overridden. Everything else
/// keeps its production default and is expected to be the compose stack.
fn args_for(database_url: &str, redis_url: &str, issuer_url: &str) -> Vec<String> {
    let db = url::Url::parse(database_url).expect("DATABASE_URL is a url");

    vec![
        "api".to_owned(),
        "--database-host".to_owned(),
        db.host_str().unwrap_or("localhost").to_owned(),
        "--database-port".to_owned(),
        db.port().unwrap_or(5432).to_string(),
        "--database-user".to_owned(),
        db.username().to_owned(),
        "--database-password".to_owned(),
        db.password().unwrap_or_default().to_owned(),
        "--database-name".to_owned(),
        db.path().trim_start_matches('/').to_owned(),
        "--rate-limit-redis-url".to_owned(),
        redis_url.to_owned(),
        // The limiter keys on client IP, so the whole suite shares one window
        // across runs. The production default fails a second consecutive run.
        "--rate-limit-per-minute".to_owned(),
        "100000".to_owned(),
        "--auth-issuer".to_owned(),
        issuer_url.to_owned(),
        "--file-storage-auto-create-bucket".to_owned(),
        "false".to_owned(),
    ]
}

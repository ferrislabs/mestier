//! Brings up the quote API on a real socket, against a real database.
//!
//! `AppState` is built by `handlers::state`, the same function the binary
//! calls, so the test exercises the production wiring rather than a
//! reconstruction of it. Only two things are pointed elsewhere: the auth
//! issuer, at the fake JWKS server, and the database, at whatever `DATABASE_URL`
//! names.

use std::{net::SocketAddr, sync::Arc};

use args::Args;
use clap::Parser;
use mestier_core::Permissions;
use sqlx::PgPool;
use uuid::Uuid;

use crate::issuer;

/// What a scenario needs to address the API it just started.
pub struct App {
    pub base_url: String,
    pub token: String,
    pub pool: PgPool,
    pub organization_id: Uuid,
    pub customer_id: Uuid,
    pub customer_context_id: Uuid,
    /// A second member of the same organization, holding membership but no
    /// role assignment at all — #305's `quote.manage` gate refuses this one
    /// outright on every write.
    pub no_role_token: String,
    user_id: Uuid,
    no_role_user_id: Uuid,
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

    let router = handlers_quote::router(&state).with_state(state);
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
        no_role_token: issuer::mint(&fixture.no_role_sub),
        pool,
        organization_id: fixture.organization_id,
        customer_id: fixture.customer_id,
        customer_context_id: fixture.customer_context_id,
        user_id: fixture.user_id,
        no_role_user_id: fixture.no_role_user_id,
    }
}

impl App {
    pub fn quotes_url(&self) -> String {
        format!(
            "{}/api/v1/organizations/{}/quotes",
            self.base_url, self.organization_id
        )
    }

    pub fn plan_proposal_url(&self, quote_id: &str) -> String {
        format!("{}/api/v1/quotes/{quote_id}/plan-proposal", self.base_url)
    }

    pub fn plan_url(&self, quote_id: &str) -> String {
        format!("{}/api/v1/quotes/{quote_id}/plan", self.base_url)
    }

    /// Drops the fixture. `org_id`/`customer_id`/`quote_id` foreign keys are
    /// `NO ACTION`, not `CASCADE` (checked directly against the live schema),
    /// so tasks and projects the plan endpoint created have to go before the
    /// organization does — explicit and ordered rather than relying on a
    /// cascade that isn't actually there, mirroring
    /// `handlers-planning`'s own harness.
    pub async fn cleanup(&self) {
        for statement in [
            "DELETE FROM tasks WHERE org_id = $1",
            "DELETE FROM projects WHERE org_id = $1",
            "DELETE FROM quote_lines WHERE org_id = $1",
            "DELETE FROM quotes WHERE org_id = $1",
            "DELETE FROM customer_contexts WHERE customer_id IN (SELECT id FROM customers WHERE org_id = $1)",
            "DELETE FROM customers WHERE org_id = $1",
            "DELETE FROM organization_members WHERE organization_id = $1",
            "DELETE FROM organizations WHERE id = $1",
        ] {
            let _ = sqlx::query(statement)
                .bind(self.organization_id)
                .execute(&self.pool)
                .await;
        }

        for user_id in [self.user_id, self.no_role_user_id] {
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
    no_role_sub: String,
    no_role_user_id: Uuid,
    organization_id: Uuid,
    customer_id: Uuid,
    customer_context_id: Uuid,
}

/// The smallest graph a quote needs: a user, the organization they belong to,
/// and a customer with a context to bill against.
///
/// The user is seeded rather than left to the auth middleware, which creates
/// one from the token but hands back no id, and the membership row needs one.
/// Queries are unchecked on purpose: `query!` would demand a regenerated
/// `.sqlx` cache for statements that only ever run against a live database.
async fn seed(pool: &PgPool) -> Fixture {
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

    let organization_id = Uuid::now_v7();
    sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
        .bind(organization_id)
        .bind("Test Org")
        .bind(format!("test-org-{organization_id}"))
        .bind(user_id)
        .execute(pool)
        .await
        .expect("seed the organization");

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

    // #305: every write on the quote API now gates on `quote.manage`. The
    // fixture's main caller (`app.token`) needs it, or every existing test
    // in this suite would start seeing 403 instead of the answer it asserts
    // on — `Permissions::ALL` keeps that unchanged, the same choice
    // `handlers-reporting`'s harness made for its own main caller.
    let owner_role_id = seed_role(pool, organization_id, "test-owner", Permissions::ALL.0).await;
    assign_role(pool, member_id, owner_role_id).await;

    // A second member with membership but no role assignment at all — the
    // bare case `quote.manage` now refuses outright on a write.
    let no_role_user_id = Uuid::now_v7();
    let no_role_sub = format!("sub-e2e-{no_role_user_id}");
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(no_role_user_id)
    .bind(format!("artisan-{no_role_user_id}@example.com"))
    .bind(format!("artisan-{no_role_user_id}"))
    .bind("Artisan No Role")
    .bind(&no_role_sub)
    .execute(pool)
    .await
    .expect("seed the no-role user");

    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name) VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::now_v7())
    .bind(organization_id)
    .bind(no_role_user_id)
    .bind("Artisan No Role")
    .execute(pool)
    .await
    .expect("seed the no-role membership");

    let customer_id = Uuid::now_v7();
    sqlx::query("INSERT INTO customers (id, org_id, name) VALUES ($1, $2, $3)")
        .bind(customer_id)
        .bind(organization_id)
        .bind("Duval Masonry")
        .execute(pool)
        .await
        .expect("seed the customer");

    let customer_context_id = Uuid::now_v7();
    sqlx::query("INSERT INTO customer_contexts (id, customer_id, label) VALUES ($1, $2, $3)")
        .bind(customer_context_id)
        .bind(customer_id)
        .bind("Main site")
        .execute(pool)
        .await
        .expect("seed the customer context");

    Fixture {
        sub,
        user_id,
        no_role_sub,
        no_role_user_id,
        organization_id,
        customer_id,
        customer_context_id,
    }
}

/// A role carrying exactly the given bits — mirrors
/// `handlers-reporting`'s own harness helper of the same name.
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

/// Only the three endpoints the test controls are overridden. Everything else,
/// object storage included, keeps its production default and is expected to be
/// the compose stack: `create_service` creates its bucket at startup, and that
/// call is part of what this test proves works.
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
        "--auth-issuer".to_owned(),
        issuer_url.to_owned(),
        // None of these suites touch object storage, but `create_service`
        // creates the bucket at startup by default, which turned a reachable S3
        // into a prerequisite for running them at all. Saying no here drops
        // `rustfs` from the list — and is what lets them run in CI against two
        // service containers instead of three.
        "--file-storage-auto-create-bucket".to_owned(),
        "false".to_owned(),
    ]
}

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
    user_id: Uuid,
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
        pool,
        organization_id: fixture.organization_id,
        customer_id: fixture.customer_id,
        customer_context_id: fixture.customer_context_id,
        user_id: fixture.user_id,
    }
}

impl App {
    pub fn quotes_url(&self) -> String {
        format!(
            "{}/api/v1/organizations/{}/quotes",
            self.base_url, self.organization_id
        )
    }

    /// Drops the fixture. Deleting the organization cascades to its quotes and
    /// customers; the user row outlives it and goes separately.
    pub async fn cleanup(&self) {
        for statement in [
            "DELETE FROM organizations WHERE id = $1",
            "DELETE FROM users WHERE id = $2",
        ] {
            let _ = sqlx::query(statement)
                .bind(self.organization_id)
                .bind(self.user_id)
                .execute(&self.pool)
                .await;
        }
    }
}

struct Fixture {
    sub: String,
    user_id: Uuid,
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

    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name) VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::now_v7())
    .bind(organization_id)
    .bind(user_id)
    .bind("Artisan Test")
    .execute(pool)
    .await
    .expect("seed the membership");

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
        organization_id,
        customer_id,
        customer_context_id,
    }
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

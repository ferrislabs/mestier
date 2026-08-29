//! Brings up the purchasing API on a real socket, against a real database.
//!
//! Same shape as `handlers-invoice`'s own harness: `AppState` is built by
//! `handlers::state`, the same function the binary calls, and only the auth
//! issuer and the database are pointed elsewhere.
//!
//! Lighter fixture than the invoice suite's: a supplier invoice never issues
//! anything and needs no legal identity, no customer, no quote — just a
//! user, their organization, and (for the allocation routes) a project to
//! attribute cost to.

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
    user_id: Uuid,
}

/// Panics rather than skipping when the stack is down: the test is
/// `#[ignore]`d, so reaching this function is already a statement that the
/// stack is up.
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

    let router = handlers_purchase::router(&state).with_state(state);
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
        user_id: fixture.user_id,
    }
}

impl App {
    pub fn supplier_invoices_url(&self) -> String {
        format!(
            "{}/api/v1/organizations/{}/supplier-invoices",
            self.base_url, self.organization_id
        )
    }

    pub fn import_url(&self) -> String {
        format!("{}/import", self.supplier_invoices_url())
    }

    pub fn supplier_invoice_url(&self, supplier_invoice_id: &str) -> String {
        format!(
            "{}/api/v1/supplier-invoices/{supplier_invoice_id}",
            self.base_url
        )
    }

    pub fn line_allocations_url(&self, supplier_invoice_line_id: &str) -> String {
        format!(
            "{}/api/v1/supplier-invoice-lines/{supplier_invoice_line_id}/allocations",
            self.base_url
        )
    }

    pub fn project_supplier_costs_url(&self, project_id: &str) -> String {
        format!(
            "{}/api/v1/projects/{project_id}/supplier-costs",
            self.base_url
        )
    }

    /// A bare project for the seeded organization — an allocation's only
    /// requirement, no customer/quote attached.
    pub async fn seed_project(&self) -> Uuid {
        let project_id = Uuid::now_v7();
        sqlx::query("INSERT INTO projects (id, org_id, name) VALUES ($1, $2, $3)")
            .bind(project_id)
            .bind(self.organization_id)
            .bind("Chantier de test")
            .execute(&self.pool)
            .await
            .expect("seed the project");

        project_id
    }

    /// Drops the fixture. Best-effort: foreign keys from supplier invoices
    /// and projects are plain `REFERENCES`, not `ON DELETE CASCADE`, so a
    /// run that left rows behind would otherwise fail this cleanup —
    /// swallowing the error is the same choice `handlers-invoice`'s own
    /// harness makes.
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
}

/// The smallest graph a supplier invoice needs: a user, and the organization
/// they belong to.
///
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

    Fixture {
        sub,
        user_id,
        organization_id,
    }
}

/// Only the endpoints the test controls are overridden. Object storage
/// itself keeps its production default (`http://localhost:9000`,
/// `rustfsadmin`/`rustfsadmin`, bucket `mestier-files`) and is expected to
/// be the compose stack, or CI's own `rustfs` service.
///
/// Unlike `handlers-invoice`'s own harness, `auto_create_bucket` is left at
/// its default of `true` rather than forced to `false`: this is the first
/// e2e suite whose handler actually calls `file_storage.upload`, and a
/// freshly started `rustfs` container (CI's, in particular) has no bucket
/// yet for `import` to write into.
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
        // The rate limiter keys on client IP alone, and every test in this
        // suite calls in from the same loopback address through the same
        // Redis — so the sliding window is shared across every test in a
        // run, and across a run and the one before it if run twice inside
        // the same window. The production default of 120/minute is a
        // limit on one real caller, not on an entire suite's worth of
        // fixtures; a value that low turned a second consecutive run of a
        // clean suite into a false failure.
        "--rate-limit-per-minute".to_owned(),
        "100000".to_owned(),
        "--auth-issuer".to_owned(),
        issuer_url.to_owned(),
    ]
}

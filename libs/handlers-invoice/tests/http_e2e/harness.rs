//! Brings up the invoice API on a real socket, against a real database.
//!
//! Same shape as `handlers-quote`'s own harness: `AppState` is built by
//! `handlers::state`, the same function the binary calls, and only the auth
//! issuer and the database are pointed elsewhere.
//!
//! Richer fixture than the quote suite's, though: most of this crate's
//! routes need a project (with or without a quote) on top of the
//! customer/context pair, and — because issuing anything at all goes
//! through `LegalIdentity::try_from_organization` — a *complete* legal
//! identity, seeded directly here so only the one test that is actually
//! about #310's refusal has to deal with an incomplete one.

use std::{net::SocketAddr, sync::Arc};

use args::Args;
use chrono::{DateTime, SubsecRound, Utc};
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

/// Postgres stores microseconds; `Utc::now()` gives nanoseconds on Linux.
/// Mismatched instants after a round trip have broken CI four times in
/// this repo already (see `libs/handlers-field/tests/http_e2e/harness.rs`
/// and `mestier_core::application::test_support::now_storable`, neither of
/// which this crate can import: one is a different crate's test-only
/// helper, the other is crate-private).
pub fn now_storable() -> DateTime<Utc> {
    Utc::now().trunc_subsecs(6)
}

/// Panics rather than skipping when the stack is down: the test is `#[ignore]`d,
/// so reaching this function is already a statement that the stack is up.
///
/// Seeds a *complete* legal identity: almost every route in this crate
/// issues something, and issuing goes through
/// `LegalIdentity::try_from_organization` regardless of which act triggers
/// it, so an incomplete identity would block everything but the plainest
/// draft CRUD. Only `pdf_export_refuses_an_incomplete_legal_identity_naming_the_missing_fields`
/// needs the bare version, via `start_with_incomplete_identity`.
pub async fn start() -> App {
    bootstrap(true).await
}

pub async fn start_with_incomplete_identity() -> App {
    bootstrap(false).await
}

async fn bootstrap(complete_legal_identity: bool) -> App {
    let issuer_url = issuer::spawn();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set to run the http end-to-end tests");
    let redis_url = std::env::var("RATE_LIMIT_REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_owned());

    let pool = PgPool::connect(&database_url)
        .await
        .expect("connect to the test database");
    let fixture = seed(&pool, complete_legal_identity).await;

    let args = Arc::new(Args::parse_from(args_for(
        &database_url,
        &redis_url,
        &issuer_url,
    )));
    let state = handlers::state(args)
        .await
        .expect("build AppState for the test");

    let router = handlers_invoice::router(&state).with_state(state);
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
    pub fn invoices_url(&self) -> String {
        format!(
            "{}/api/v1/organizations/{}/invoices",
            self.base_url, self.organization_id
        )
    }

    pub fn invoice_url(&self, invoice_id: &str) -> String {
        format!("{}/api/v1/invoices/{invoice_id}", self.base_url)
    }

    /// A quote for `net_cents` net, already `ACCEPTED` — a deposit/final
    /// invoice needs an accepted total to bill against. `gross_cents` is
    /// set equal to `net_cents`: the seeded organization is not subject to
    /// VAT (see `bootstrap`'s legal identity), so every real total in this
    /// suite is VAT-free by construction and the two must agree.
    pub async fn seed_quote(&self, net_cents: i32) -> Uuid {
        let quote_id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO quotes (id, org_id, customer_id, customer_context_id, status, \
             net_cents, gross_cents, reference, title) \
             VALUES ($1, $2, $3, $4, 'ACCEPTED', $5, $5, $6, $7)",
        )
        .bind(quote_id)
        .bind(self.organization_id)
        .bind(self.customer_id)
        .bind(self.customer_context_id)
        .bind(net_cents)
        .bind(format!("DEV-TEST-{quote_id}"))
        .bind("Devis de test")
        .execute(&self.pool)
        .await
        .expect("seed the quote");

        quote_id
    }

    /// A project for the seeded customer, optionally pointed at a quote —
    /// `issue_deposit`/`issue_final_invoice`/the billing summary all need
    /// one with a quote, `create_invoice`'s own `project_id` works with
    /// either.
    pub async fn seed_project(&self, quote_id: Option<Uuid>) -> Uuid {
        let project_id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO projects (id, org_id, name, customer_id, customer_context_id, quote_id) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(project_id)
        .bind(self.organization_id)
        .bind("Chantier de test")
        .bind(self.customer_id)
        .bind(self.customer_context_id)
        .bind(quote_id)
        .execute(&self.pool)
        .await
        .expect("seed the project");

        project_id
    }

    /// Drops the fixture. Best-effort: `org_id`/`customer_id` foreign keys
    /// from quotes/projects/invoices are plain `REFERENCES`, not
    /// `ON DELETE CASCADE`, so a run that left rows behind would otherwise
    /// fail this cleanup — swallowing the error is the same choice
    /// `handlers-quote`'s own harness makes.
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

/// The smallest graph an invoice needs: a user, the organization they
/// belong to (with or without a complete legal identity), and a customer
/// with a context to bill against.
///
/// Queries are unchecked on purpose: `query!` would demand a regenerated
/// `.sqlx` cache for statements that only ever run against a live database.
async fn seed(pool: &PgPool, complete_legal_identity: bool) -> Fixture {
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

    if complete_legal_identity {
        sqlx::query(
            r#"UPDATE organizations SET
                legal_name = $2,
                legal_form = $3,
                registration_number = $4,
                vat_status = $5,
                vat_exemption_basis = $6,
                insurance_mention = $7,
                address_line1 = $8,
                address_postal_code = $9,
                address_city = $10,
                address_country = $11
            WHERE id = $1"#,
        )
        .bind(organization_id)
        .bind("Acme SARL")
        .bind("SARL")
        .bind("123 456 789 00012")
        .bind("not_subject")
        .bind("Article 293 B du CGI")
        .bind("RC Pro n. 123456 - MAAF Assurances")
        .bind("12 rue des Artisans")
        .bind("75001")
        .bind("Paris")
        .bind("FR")
        .execute(pool)
        .await
        .expect("seed the legal identity");
    }

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
/// the compose stack.
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
        "--file-storage-auto-create-bucket".to_owned(),
        "false".to_owned(),
    ]
}

//! Brings the field API up on a real socket, against a real database.
//!
//! `AppState` comes from `handlers::state`, the same function the binary
//! calls, so the test exercises the production wiring. Only the auth issuer is
//! pointed elsewhere, at a local JWKS server.

use std::{net::SocketAddr, sync::Arc};

use args::Args;
use chrono::{DateTime, Duration, Utc};
use clap::Parser;
use sqlx::PgPool;
use uuid::Uuid;

use crate::issuer;

pub struct App {
    pub base_url: String,
    pub token: String,
    pub pool: PgPool,
    pub organization_id: Uuid,
    /// A customer project with a quote: one three-hour task and one expense.
    pub project_id: Uuid,
    /// A project with no customer and no quote — the case the old clocked model
    /// could not express at all.
    pub internal_project_id: Uuid,
    /// A project whose only task sits two months back, so the period must not
    /// report it.
    pub stale_project_id: Uuid,
    user_id: Uuid,
    other_user_id: Uuid,
}

pub async fn start() -> App {
    let issuer_url = issuer::spawn();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set to run the field end-to-end tests");
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

    let router = handlers_reporting::router(&state).with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind the test api");
    let addr = listener.local_addr().expect("read the test api address");
    tokio::spawn(async move {
        // `ConnectInfo` the way the binary supplies it: the rate-limit
        // middleware keys on the peer address and 500s without it.
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
        project_id: fixture.project_id,
        internal_project_id: fixture.internal_project_id,
        stale_project_id: fixture.stale_project_id,
        user_id: fixture.user_id,
        other_user_id: fixture.other_user_id,
    }
}

impl App {
    pub fn url(&self, suffix: &str) -> String {
        format!(
            "{}/api/v1/organizations/{}/reporting{suffix}",
            self.base_url, self.organization_id
        )
    }

    /// Removes the fixture, child rows first.
    ///
    /// Explicit and ordered rather than relying on cascades: `employees`,
    /// `automation.event` and others reference `organizations` with a plain
    /// foreign key, so a single delete is refused. Errors are raised, not
    /// swallowed: a cleanup that fails quietly is how a shared development
    /// database fills up with fixtures, which is what this suite found.
    pub async fn cleanup(&self) {
        for statement in [
            "DELETE FROM automation.event WHERE org_id = $1",
            "DELETE FROM time_entries WHERE org_id = $1",
            "DELETE FROM day_logs WHERE org_id = $1",
            "DELETE FROM tasks WHERE org_id = $1",
            "DELETE FROM projects WHERE org_id = $1",
            "DELETE FROM quotes WHERE org_id = $1",
            "DELETE FROM employees WHERE org_id = $1",
            "DELETE FROM organization_members WHERE organization_id = $1",
            "DELETE FROM customer_contexts WHERE customer_id IN (SELECT id FROM customers WHERE org_id = $1)",
            "DELETE FROM customers WHERE org_id = $1",
            "DELETE FROM organizations WHERE id = $1",
        ] {
            sqlx::query(statement)
                .bind(self.organization_id)
                .execute(&self.pool)
                .await
                .unwrap_or_else(|e| panic!("cleanup failed on `{statement}`: {e}"));
        }

        for user_id in [self.user_id, self.other_user_id] {
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&self.pool)
                .await
                .expect("clear the fixture user");
        }
    }
}

struct Fixture {
    sub: String,
    user_id: Uuid,
    other_user_id: Uuid,
    organization_id: Uuid,
    project_id: Uuid,
    internal_project_id: Uuid,
    stale_project_id: Uuid,
}

/// Three projects and two people, with nothing clocked anywhere.
///
/// Anchored on yesterday at fixed hours rather than on `Utc::now()` plus an
/// offset: the cost now comes from the task's own window, so a fixture running
/// near midnight would have its window clipped by the period and the numbers
/// below would stop being stateable.
async fn seed(pool: &PgPool) -> Fixture {
    let organization_id = Uuid::now_v7();
    let (user_id, sub, member_id, employee_id) =
        seed_person(pool, organization_id, true, CostBasis::Hourly).await;
    // Salaried, on the exact shape that read as 0,00 €: 3 500 € a month on a
    // 35 h contract, which is 23,08 € an hour.
    let (other_user_id, _, other_member_id, _) =
        seed_person(pool, organization_id, false, CostBasis::Salaried).await;
    let _ = employee_id;

    let customer_id = Uuid::now_v7();
    sqlx::query("INSERT INTO customers (id, org_id, name) VALUES ($1, $2, $3)")
        .bind(customer_id)
        .bind(organization_id)
        .bind("Duval Masonry")
        .execute(pool)
        .await
        .expect("seed the customer");

    let yesterday = (Utc::now() - Duration::days(1))
        .date_naive()
        .and_hms_opt(9, 0, 0)
        .expect("09:00 exists")
        .and_utc();

    let quote_id = seed_quote(pool, organization_id, customer_id).await;
    let project_id = seed_project(
        pool,
        organization_id,
        "Entretien annuel",
        Some(customer_id),
        Some(quote_id),
    )
    .await;
    let internal_project_id = seed_project(pool, organization_id, "Vie interne", None, None).await;
    let stale_project_id = seed_project(
        pool,
        organization_id,
        "Chantier terminé",
        Some(customer_id),
        None,
    )
    .await;

    // Three hours for one person, plus 45 euros of travel on the same task.
    seed_task(
        pool,
        organization_id,
        project_id,
        &[member_id],
        yesterday,
        yesterday + Duration::hours(3),
        "Taille de haie",
        4_500,
        Some("Déplacement Clermont"),
    )
    .await;

    // Two hours with two people on it: the six-person-hour meeting the clocked
    // model reported as nothing at all.
    seed_task(
        pool,
        organization_id,
        internal_project_id,
        &[member_id, other_member_id],
        yesterday + Duration::hours(5),
        yesterday + Duration::hours(7),
        "Réunion hebdo",
        0,
        None,
    )
    .await;

    // Outside the period entirely.
    seed_task(
        pool,
        organization_id,
        stale_project_id,
        &[member_id],
        yesterday - Duration::days(60),
        yesterday - Duration::days(60) + Duration::hours(4),
        "Chantier de printemps",
        0,
        None,
    )
    .await;

    // Nine hours clocked, to prove the report ignores them. Nine rather than a
    // number the plan also produces: a coincidence would make the assertion
    // pass for the wrong reason.
    seed_stray_time_entry(pool, organization_id, project_id, employee_id, yesterday).await;

    Fixture {
        sub,
        user_id,
        other_user_id,
        organization_id,
        project_id,
        internal_project_id,
        stale_project_id,
    }
}

/// A user, their organization seat, and the employee profile that carries the
/// hourly rate. The field routes refuse a member without that profile, so all
/// three rows are needed for the caller to get past authorization.
/// How the seeded person is costed. Both bases are exercised because the report
/// has to agree with itself across them.
enum CostBasis {
    Hourly,
    Salaried,
}

async fn seed_person(
    pool: &PgPool,
    organization_id: Uuid,
    owns_organization: bool,
    cost_basis: CostBasis,
) -> (Uuid, String, Uuid, Uuid) {
    let user_id = Uuid::now_v7();
    let sub = format!("sub-field-{user_id}");
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("worker-{user_id}@example.com"))
    .bind(format!("worker-{user_id}"))
    .bind("Field Worker")
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
    .bind("Worker")
    .execute(pool)
    .await
    .expect("seed the membership");

    let employee_id = Uuid::now_v7();
    let (hourly_rate_cents, is_salaried, monthly_cost_cents) = match cost_basis {
        CostBasis::Hourly => (Some(3_500_i32), false, None),
        CostBasis::Salaried => (None, true, Some(350_000_i32)),
    };
    sqlx::query(
        "INSERT INTO employees (id, org_id, member_id, hourly_rate_cents, is_salaried, monthly_cost_cents, weekly_contract_minutes)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(employee_id)
    .bind(organization_id)
    .bind(member_id)
    .bind(hourly_rate_cents)
    .bind(is_salaried)
    .bind(monthly_cost_cents)
    .bind(2100)
    .execute(pool)
    .await
    .expect("seed the employee profile");

    (user_id, sub, member_id, employee_id)
}

async fn seed_quote(pool: &PgPool, organization_id: Uuid, customer_id: Uuid) -> Uuid {
    let context_id = Uuid::now_v7();
    sqlx::query("INSERT INTO customer_contexts (id, customer_id, label) VALUES ($1, $2, $3)")
        .bind(context_id)
        .bind(customer_id)
        .bind("Chantier principal")
        .execute(pool)
        .await
        .expect("seed the customer context");

    let quote_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO quotes (id, org_id, customer_id, customer_context_id, reference, title, status, net_cents, gross_cents)
         VALUES ($1, $2, $3, $4, $5, $6, CAST($7 AS text)::quote_status, $8, $8)",
    )
    .bind(quote_id)
    .bind(organization_id)
    .bind(customer_id)
    .bind(context_id)
    .bind(format!("DEV-TEST-{quote_id}"))
    .bind("Entretien annuel")
    .bind("ACCEPTED")
    .bind(420_000_i32)
    .execute(pool)
    .await
    .expect("seed the quote");

    quote_id
}

async fn seed_project(
    pool: &PgPool,
    organization_id: Uuid,
    name: &str,
    customer_id: Option<Uuid>,
    quote_id: Option<Uuid>,
) -> Uuid {
    let project_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO projects (id, org_id, name, customer_id, quote_id) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(project_id)
    .bind(organization_id)
    .bind(name)
    .bind(customer_id)
    .bind(quote_id)
    .execute(pool)
    .await
    .expect("seed the project");

    project_id
}

/// Time clocked on the task, which nothing reads for money any more.
async fn seed_stray_time_entry(
    pool: &PgPool,
    organization_id: Uuid,
    task_id: Uuid,
    employee_id: Uuid,
    anchor: DateTime<Utc>,
) {
    let task_id: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM tasks WHERE org_id = $1 AND project_id = $2 LIMIT 1")
            .bind(organization_id)
            .bind(task_id)
            .fetch_optional(pool)
            .await
            .expect("look up the task to clock against");

    let Some(task_id) = task_id else {
        return;
    };

    sqlx::query(
        "INSERT INTO time_entries (id, org_id, task_id, employee_id, started_at, ended_at)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(Uuid::now_v7())
    .bind(organization_id)
    .bind(task_id)
    .bind(employee_id)
    .bind(anchor)
    .bind(anchor + Duration::hours(9))
    .execute(pool)
    .await
    .expect("seed the stray time entry");
}

#[allow(clippy::too_many_arguments)]
async fn seed_task(
    pool: &PgPool,
    organization_id: Uuid,
    project_id: Uuid,
    member_ids: &[Uuid],
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    title: &str,
    expenses_cents: i32,
    expenses_label: Option<&str>,
) -> Uuid {
    let task_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO tasks (id, org_id, project_id, starts_at, ends_at, all_day, status, title, expenses_cents, expenses_label)
         VALUES ($1, $2, $3, $4, $5, false, CAST($6 AS text)::task_status, $7, $8, $9)",
    )
    .bind(task_id)
    .bind(organization_id)
    .bind(project_id)
    .bind(starts_at)
    .bind(ends_at)
    .bind("PLANNED")
    .bind(title)
    .bind(expenses_cents)
    .bind(expenses_label)
    .execute(pool)
    .await
    .expect("seed the task");

    for member_id in member_ids {
        sqlx::query(
            "INSERT INTO task_assignments (id, org_id, task_id, member_id) VALUES ($1, $2, $3, $4)",
        )
        .bind(Uuid::now_v7())
        .bind(organization_id)
        .bind(task_id)
        .bind(member_id)
        .execute(pool)
        .await
        .expect("seed the assignment");
    }

    task_id
}

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

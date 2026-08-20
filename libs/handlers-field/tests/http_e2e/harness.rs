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
    pub task_id: Uuid,
    /// A second employee's job, used to prove one worker cannot touch another's.
    pub other_task_id: Uuid,
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

    let router = handlers_field::router(&state).with_state(state);
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
        task_id: fixture.task_id,
        other_task_id: fixture.other_task_id,
        user_id: fixture.user_id,
        other_user_id: fixture.other_user_id,
    }
}

impl App {
    /// The user the token authenticates as, for fixtures that need their employee.
    pub fn owner_user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn url(&self, suffix: &str) -> String {
        format!(
            "{}/api/v1/organizations/{}/field{suffix}",
            self.base_url, self.organization_id
        )
    }

    pub fn entry_url(&self, entry_id: &str, suffix: &str) -> String {
        format!(
            "{}/api/v1/field/time-entries/{entry_id}{suffix}",
            self.base_url
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
            "DELETE FROM employees WHERE org_id = $1",
            "DELETE FROM organization_members WHERE organization_id = $1",
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

/// An entry begun yesterday and never closed: the forgotten clock-off.
pub async fn seed_forgotten_entry(pool: &PgPool, app: &App) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO time_entries (id, org_id, task_id, employee_id, started_at, ended_at)
         VALUES ($1, $2, $3, (SELECT e.id FROM employees e
                              JOIN organization_members m ON m.id = e.member_id
                              JOIN users u ON u.id = m.user_id
                              WHERE u.id = $4), $5, NULL)",
    )
    .bind(id)
    .bind(app.organization_id)
    .bind(app.task_id)
    .bind(app.owner_user_id())
    .bind(Utc::now() - Duration::hours(30))
    .execute(pool)
    .await
    .expect("seed the forgotten entry");

    id
}

struct Fixture {
    sub: String,
    user_id: Uuid,
    other_user_id: Uuid,
    organization_id: Uuid,
    task_id: Uuid,
    other_task_id: Uuid,
}

/// Two employees, each with a job today. The second exists so the suite can
/// show that one worker's routes cannot reach the other's entry.
async fn seed(pool: &PgPool) -> Fixture {
    let organization_id = Uuid::now_v7();
    let (user_id, sub, member_id, employee_id) = seed_person(pool, organization_id, true).await;
    let (other_user_id, _, other_member_id, _) = seed_person(pool, organization_id, false).await;
    let _ = employee_id;

    let customer_id = Uuid::now_v7();
    sqlx::query("INSERT INTO customers (id, org_id, name) VALUES ($1, $2, $3)")
        .bind(customer_id)
        .bind(organization_id)
        .bind("Duval Masonry")
        .execute(pool)
        .await
        .expect("seed the customer");

    let now = Utc::now();
    let task_id = seed_task(pool, organization_id, customer_id, member_id, now).await;
    let other_task_id = seed_task(pool, organization_id, customer_id, other_member_id, now).await;

    Fixture {
        sub,
        user_id,
        other_user_id,
        organization_id,
        task_id,
        other_task_id,
    }
}

/// A user, their organization seat, and the employee profile that carries the
/// hourly rate. The field routes refuse a member without that profile, so all
/// three rows are needed for the caller to get past authorization.
async fn seed_person(
    pool: &PgPool,
    organization_id: Uuid,
    owns_organization: bool,
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
    sqlx::query(
        "INSERT INTO employees (id, org_id, member_id, hourly_rate_cents, weekly_contract_minutes) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(employee_id)
    .bind(organization_id)
    .bind(member_id)
    .bind(3500)
    .bind(2100)
    .execute(pool)
    .await
    .expect("seed the employee profile");

    (user_id, sub, member_id, employee_id)
}

async fn seed_task(
    pool: &PgPool,
    organization_id: Uuid,
    customer_id: Uuid,
    member_id: Uuid,
    now: DateTime<Utc>,
) -> Uuid {
    let task_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO tasks (id, org_id, customer_id, starts_at, ends_at, all_day, status, title)
         VALUES ($1, $2, $3, $4, $5, false, CAST($6 AS text)::task_status, $7)",
    )
    .bind(task_id)
    .bind(organization_id)
    .bind(customer_id)
    .bind(now - Duration::hours(1))
    .bind(now + Duration::hours(4))
    .bind("PLANNED")
    .bind("Taille de haie")
    .execute(pool)
    .await
    .expect("seed the task");

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
    ]
}

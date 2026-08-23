//! Brings the reference API up on a real socket, against a real database.
//!
//! `AppState` comes from `handlers::state`, the same function the binary
//! calls, so the test exercises the production wiring. Only the auth issuer is
//! pointed elsewhere, at a local JWKS server.

use std::{net::SocketAddr, sync::Arc};

use args::Args;
use chrono::NaiveDate;
use clap::Parser;
use sqlx::PgPool;
use uuid::Uuid;

use crate::issuer;

pub struct App {
    pub base_url: String,
    pub token: String,
    pub pool: PgPool,
    pub organization_id: Uuid,
    pub employee_id: Uuid,
    /// A cost basis version already on file for `employee_id`, open-ended —
    /// what the fixture starts every test from.
    pub cost_basis_id: Uuid,
    /// A second organization's cost basis version, for the cross-tenant
    /// refusal test: `token` above must never be able to read or correct it.
    pub other_cost_basis_id: Uuid,
    pub other_employee_id: Uuid,
    user_id: Uuid,
    other_user_id: Uuid,
}

pub async fn start() -> App {
    let issuer_url = issuer::spawn();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set to run the reference end-to-end tests");
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

    let router = handlers_reference::router(&state).with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind the test api");
    let addr = listener.local_addr().expect("read the test api address");
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
        employee_id: fixture.employee_id,
        cost_basis_id: fixture.cost_basis_id,
        other_cost_basis_id: fixture.other_cost_basis_id,
        other_employee_id: fixture.other_employee_id,
        user_id: fixture.user_id,
        other_user_id: fixture.other_user_id,
    }
}

impl App {
    pub fn url(&self, suffix: &str) -> String {
        format!("{}/api/v1{suffix}", self.base_url)
    }

    /// Removes both organizations seeded by this fixture, child rows first.
    pub async fn cleanup(&self) {
        for org_id in [self.organization_id, self.other_organization_id().await] {
            for statement in [
                "DELETE FROM employee_cost_bases WHERE org_id = $1",
                "DELETE FROM employees WHERE org_id = $1",
                "DELETE FROM organization_members WHERE organization_id = $1",
                "DELETE FROM organizations WHERE id = $1",
            ] {
                sqlx::query(statement)
                    .bind(org_id)
                    .execute(&self.pool)
                    .await
                    .unwrap_or_else(|e| panic!("cleanup failed on `{statement}`: {e}"));
            }
        }

        for user_id in [self.user_id, self.other_user_id] {
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&self.pool)
                .await
                .expect("clear the fixture user");
        }
    }

    /// Looked up rather than stored: the second organization only exists to
    /// own `other_cost_basis_id`, and cleanup is the only caller that needs
    /// its id.
    async fn other_organization_id(&self) -> Uuid {
        sqlx::query_scalar("SELECT org_id FROM employee_cost_bases WHERE id = $1")
            .bind(self.other_cost_basis_id)
            .fetch_one(&self.pool)
            .await
            .expect("find the other organization via its cost basis")
    }
}

struct Fixture {
    sub: String,
    user_id: Uuid,
    other_user_id: Uuid,
    organization_id: Uuid,
    employee_id: Uuid,
    cost_basis_id: Uuid,
    other_cost_basis_id: Uuid,
    other_employee_id: Uuid,
}

/// One organization with an owner and an hourly employee already on an open
/// cost basis, plus a second organization's employee — used only as the
/// target of the cross-tenant refusal test, never reachable through the
/// first token.
async fn seed(pool: &PgPool) -> Fixture {
    let organization_id = Uuid::now_v7();
    let (user_id, sub, employee_id, cost_basis_id) =
        seed_org_owner_and_employee(pool, organization_id).await;

    let other_organization_id = Uuid::now_v7();
    let (other_user_id, _, other_employee_id, other_cost_basis_id) =
        seed_org_owner_and_employee(pool, other_organization_id).await;

    Fixture {
        sub,
        user_id,
        other_user_id,
        organization_id,
        employee_id,
        cost_basis_id,
        other_cost_basis_id,
        other_employee_id,
    }
}

/// A user who owns `organization_id`, their seat, an hourly employee profile
/// and its open cost basis version — the minimal graph a cost-basis route
/// needs, seeded directly rather than through `upsert_employee_profile` so
/// the fixture controls the version's own `effective_from`.
async fn seed_org_owner_and_employee(
    pool: &PgPool,
    organization_id: Uuid,
) -> (Uuid, String, Uuid, Uuid) {
    let user_id = Uuid::now_v7();
    let sub = format!("sub-reference-{user_id}");
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("owner-{user_id}@example.com"))
    .bind(format!("owner-{user_id}"))
    .bind("Reference Owner")
    .bind(&sub)
    .execute(pool)
    .await
    .expect("seed the user");

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
    .bind("Owner")
    .execute(pool)
    .await
    .expect("seed the membership");

    // `member.manage`, the bit cost-basis routes gate on: `organizations.
    // owner_id` alone carries no permissions, `create_organization` is what
    // normally seeds this role and assigns it to the owner's seat.
    const MANAGE_MEMBERS: i64 = 1 << 1;
    let role_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO roles (id, organization_id, name, permissions) VALUES ($1, $2, $3, $4)",
    )
    .bind(role_id)
    .bind(organization_id)
    .bind("Owner")
    .bind(MANAGE_MEMBERS)
    .execute(pool)
    .await
    .expect("seed the owner role");
    sqlx::query("INSERT INTO member_roles (id, member_id, role_id) VALUES ($1, $2, $3)")
        .bind(Uuid::now_v7())
        .bind(member_id)
        .bind(role_id)
        .execute(pool)
        .await
        .expect("assign the owner role");

    let employee_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO employees (id, org_id, member_id, hourly_rate_cents, weekly_contract_minutes)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(employee_id)
    .bind(organization_id)
    .bind(member_id)
    .bind(3_500_i32)
    .bind(2100)
    .execute(pool)
    .await
    .expect("seed the employee profile");

    let cost_basis_id = Uuid::now_v7();
    let far_past = NaiveDate::from_ymd_opt(2020, 1, 1).expect("a date");
    sqlx::query(
        "INSERT INTO employee_cost_bases (id, org_id, employee_id, effective_from, hourly_rate_cents, weekly_contract_minutes)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(cost_basis_id)
    .bind(organization_id)
    .bind(employee_id)
    .bind(far_past)
    .bind(3_500_i32)
    .bind(2100)
    .execute(pool)
    .await
    .expect("seed the cost basis version");

    (user_id, sub, employee_id, cost_basis_id)
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
        "--file-storage-auto-create-bucket".to_owned(),
        "false".to_owned(),
    ]
}

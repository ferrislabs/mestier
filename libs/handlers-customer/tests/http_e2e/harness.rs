//! Brings the customer API up on a real socket, against a real database.
//!
//! `AppState` comes from `handlers::state`, the same function the binary
//! calls, so the test exercises the production wiring. Only the auth issuer is
//! pointed elsewhere, at a local JWKS server.

use std::{net::SocketAddr, sync::Arc};

use args::Args;
use clap::Parser;
use mestier_core::Permissions;
use sqlx::PgPool;
use uuid::Uuid;

use crate::issuer;

pub struct App {
    pub base_url: String,
    /// Full permissions in the organization (`Permissions::ALL`) — the
    /// caller every existing assertion keeps working for.
    pub token: String,
    /// A member of the same organization holding some business permissions
    /// (`VIEW_PLANNING`) but not `MANAGE_CUSTOMERS` (#305) — the caller a
    /// customer write must refuse.
    pub restricted_token: String,
    pub pool: PgPool,
    pub organization_id: Uuid,
    user_id: Uuid,
    restricted_user_id: Uuid,
}

pub async fn start() -> App {
    let issuer_url = issuer::spawn();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set to run the customer end-to-end tests");
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

    let router = handlers_customer::router(&state).with_state(state);
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
        restricted_token: issuer::mint(&fixture.restricted_sub),
        pool,
        organization_id: fixture.organization_id,
        user_id: fixture.user_id,
        restricted_user_id: fixture.restricted_user_id,
    }
}

impl App {
    pub fn url(&self, suffix: &str) -> String {
        format!(
            "{}/api/v1/organizations/{}/customers{suffix}",
            self.base_url, self.organization_id
        )
    }

    /// Removes the fixture, child rows first.
    pub async fn cleanup(&self) {
        for statement in [
            "DELETE FROM customer_contexts WHERE customer_id IN (SELECT id FROM customers WHERE org_id = $1)",
            "DELETE FROM customer_contacts WHERE customer_id IN (SELECT id FROM customers WHERE org_id = $1)",
            "DELETE FROM customers WHERE org_id = $1",
            "DELETE FROM member_roles WHERE member_id IN (SELECT id FROM organization_members WHERE organization_id = $1)",
            "DELETE FROM roles WHERE organization_id = $1",
            "DELETE FROM organization_members WHERE organization_id = $1",
            "DELETE FROM organizations WHERE id = $1",
        ] {
            sqlx::query(statement)
                .bind(self.organization_id)
                .execute(&self.pool)
                .await
                .unwrap_or_else(|e| panic!("cleanup failed on `{statement}`: {e}"));
        }

        for user_id in [self.user_id, self.restricted_user_id] {
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
    restricted_sub: String,
    restricted_user_id: Uuid,
    organization_id: Uuid,
}

/// An organization, its owner (`Permissions::ALL`), and a second member
/// holding a role with `VIEW_PLANNING` but not `MANAGE_CUSTOMERS` — seeded
/// with raw SQL rather than through the real `create_organization` use
/// case, so the roles here are exactly the two bitmasks the test cares
/// about, not #304's defaults.
async fn seed(pool: &PgPool) -> Fixture {
    let organization_id = Uuid::now_v7();
    let (user_id, sub, owner_member_id) = seed_person(pool, organization_id, true).await;
    let (restricted_user_id, restricted_sub, restricted_member_id) =
        seed_person(pool, organization_id, false).await;

    let owner_role_id = seed_role(pool, organization_id, "test-owner", Permissions::ALL.0).await;
    assign_role(pool, owner_member_id, owner_role_id).await;

    let restricted_role_id = seed_role(
        pool,
        organization_id,
        "test-restricted",
        Permissions::VIEW_PLANNING.0,
    )
    .await;
    assign_role(pool, restricted_member_id, restricted_role_id).await;

    Fixture {
        sub,
        user_id,
        restricted_sub,
        restricted_user_id,
        organization_id,
    }
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

/// A user and their organization seat. `owns_organization` additionally
/// seeds the `organizations` row itself — needed exactly once per fixture.
async fn seed_person(
    pool: &PgPool,
    organization_id: Uuid,
    owns_organization: bool,
) -> (Uuid, String, Uuid) {
    let user_id = Uuid::now_v7();
    let sub = format!("sub-customer-{user_id}");
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("member-{user_id}@example.com"))
    .bind(format!("member-{user_id}"))
    .bind("Customer Test Member")
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
    .bind("Member")
    .execute(pool)
    .await
    .expect("seed the membership");

    (user_id, sub, member_id)
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
        // `rustfs` from the list.
        "--file-storage-auto-create-bucket".to_owned(),
        "false".to_owned(),
    ]
}

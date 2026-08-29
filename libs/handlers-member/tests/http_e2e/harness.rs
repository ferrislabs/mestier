//! Brings the member/role API up on a real socket, against a real database.
//!
//! `AppState` comes from `handlers::state`, the same function the binary
//! calls, so the test exercises the production wiring. Only the auth issuer is
//! pointed elsewhere, at a local JWKS server.

use std::{net::SocketAddr, sync::Arc};

use args::Args;
use clap::Parser;
use sqlx::PgPool;
use uuid::Uuid;

use crate::issuer;

pub struct App {
    pub base_url: String,
    pub token: String,
    pub pool: PgPool,
    pub organization_id: Uuid,
    /// The seeded `owner` role — `is_seeded`, carries `MANAGE_ROLES`.
    pub owner_role_id: Uuid,
    /// A second, ordinary member of `organization_id`, unassigned to any
    /// role — the target of the assign/list-roles tests.
    pub other_member_id: Uuid,
    /// A member of `organization_id` with no role assigned at all — #308's
    /// `role.manage`/`role.assign` gates refuse this token on every role
    /// write, while `token` above (whose seeded role carries the bits)
    /// keeps working.
    pub restricted_token: String,
    user_id: Uuid,
    restricted_user_id: Uuid,
    other_user_id: Uuid,
}

pub async fn start() -> App {
    let issuer_url = issuer::spawn();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set to run the member end-to-end tests");
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

    let router = handlers_member::router(&state).with_state(state);
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
        restricted_token: issuer::mint(&fixture.restricted_sub),
        pool,
        organization_id: fixture.organization_id,
        owner_role_id: fixture.owner_role_id,
        other_member_id: fixture.other_member_id,
        user_id: fixture.user_id,
        restricted_user_id: fixture.restricted_user_id,
        other_user_id: fixture.other_user_id,
    }
}

impl App {
    pub fn url(&self, suffix: &str) -> String {
        format!("{}/api/v1{suffix}", self.base_url)
    }

    /// Removes the fixture organization, child rows first.
    pub async fn cleanup(&self) {
        for statement in [
            "DELETE FROM member_roles WHERE role_id IN (SELECT id FROM roles WHERE organization_id = $1)",
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

        for user_id in [self.user_id, self.restricted_user_id, self.other_user_id] {
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
    other_user_id: Uuid,
    organization_id: Uuid,
    owner_role_id: Uuid,
    other_member_id: Uuid,
}

/// One organization with an owner (seeded `owner` role, `MANAGE_ROLES |
/// MANAGE_MEMBERS`), a member with no role at all (the #308 refusal case),
/// and a second ordinary member with no role — the assign-role target.
async fn seed(pool: &PgPool) -> Fixture {
    let organization_id = Uuid::now_v7();

    let user_id = Uuid::now_v7();
    let sub = format!("sub-member-{user_id}");
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("owner-{user_id}@example.com"))
    .bind(format!("owner-{user_id}"))
    .bind("Role Owner")
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

    // `MANAGE_ROLES` (1 << 2) | `MANAGE_MEMBERS` (1 << 1) — the bits #308's
    // role CRUD and assign routes gate on. `organizations.owner_id` alone
    // carries no permissions; `create_organization` is what normally seeds
    // this role and assigns it, mirrored here directly.
    const MANAGE_MEMBERS: i64 = 1 << 1;
    const MANAGE_ROLES: i64 = 1 << 2;
    let owner_role_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO roles (id, organization_id, name, permissions, is_seeded) VALUES ($1, $2, $3, $4, true)",
    )
    .bind(owner_role_id)
    .bind(organization_id)
    .bind("owner")
    .bind(MANAGE_MEMBERS | MANAGE_ROLES)
    .execute(pool)
    .await
    .expect("seed the owner role");
    sqlx::query("INSERT INTO member_roles (id, member_id, role_id) VALUES ($1, $2, $3)")
        .bind(Uuid::now_v7())
        .bind(member_id)
        .bind(owner_role_id)
        .execute(pool)
        .await
        .expect("assign the owner role");

    let restricted_user_id = Uuid::now_v7();
    let restricted_sub = format!("sub-member-no-role-{restricted_user_id}");
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(restricted_user_id)
    .bind(format!("member-{restricted_user_id}@example.com"))
    .bind(format!("member-{restricted_user_id}"))
    .bind("No Role Member")
    .bind(&restricted_sub)
    .execute(pool)
    .await
    .expect("seed the restricted user");
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name) VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::now_v7())
    .bind(organization_id)
    .bind(restricted_user_id)
    .bind("No Role")
    .execute(pool)
    .await
    .expect("seed the restricted membership");

    let other_user_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(other_user_id)
    .bind(format!("other-{other_user_id}@example.com"))
    .bind(format!("other-{other_user_id}"))
    .bind("Other Member")
    .bind(format!("sub-member-other-{other_user_id}"))
    .execute(pool)
    .await
    .expect("seed the other user");
    let other_member_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name) VALUES ($1, $2, $3, $4)",
    )
    .bind(other_member_id)
    .bind(organization_id)
    .bind(other_user_id)
    .bind("Other")
    .execute(pool)
    .await
    .expect("seed the other membership");

    Fixture {
        sub,
        user_id,
        restricted_sub,
        restricted_user_id,
        other_user_id,
        organization_id,
        owner_role_id,
        other_member_id,
    }
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

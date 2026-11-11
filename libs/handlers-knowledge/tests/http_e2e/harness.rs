use std::{net::SocketAddr, sync::Arc};

use args::Args;
use clap::Parser;
use mestier_core::Permissions;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::issuer;

pub struct Caller {
    pub token: String,
    pub user_id: Uuid,
}

pub struct App {
    pub base_url: String,
    pub pool: PgPool,
    pub organization_id: Uuid,
    pub manager: Caller,
    pub viewer: Caller,
    pub stranger: Caller,
    pub intruder: Caller,
    pub intruder_organization_id: Uuid,
    users: Vec<Uuid>,
}

pub async fn start() -> App {
    let issuer_url = issuer::spawn();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set to run the knowledge end-to-end tests");
    let redis_url = std::env::var("RATE_LIMIT_REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_owned());

    let pool = PgPool::connect(&database_url)
        .await
        .expect("connect to the test database");

    let (organization_id, owner_id) = seed_organization(&pool, "Tables Org").await;
    let manager = seed_member(&pool, organization_id, Some(Permissions::ALL.0), true).await;
    let viewer = seed_member(
        &pool,
        organization_id,
        Some(Permissions::VIEW_KNOWLEDGE.0),
        false,
    )
    .await;
    let stranger = seed_member(&pool, organization_id, None, false).await;
    let (intruder_organization_id, intruder_owner_id) =
        seed_organization(&pool, "Intruder Org").await;
    let intruder = seed_member(
        &pool,
        intruder_organization_id,
        Some(Permissions::ALL.0),
        true,
    )
    .await;

    let args = Arc::new(Args::parse_from(args_for(
        &database_url,
        &redis_url,
        &issuer_url,
    )));
    let state = handlers::state(args)
        .await
        .expect("build AppState for the test");
    let router = handlers_knowledge::router(&state).with_state(state);
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

    let users = vec![
        owner_id,
        intruder_owner_id,
        manager.user_id,
        viewer.user_id,
        stranger.user_id,
        intruder.user_id,
    ];
    App {
        base_url: format!("http://{addr}"),
        pool,
        organization_id,
        manager,
        viewer,
        stranger,
        intruder,
        intruder_organization_id,
        users,
    }
}

async fn seed_organization(pool: &PgPool, name: &str) -> (Uuid, Uuid) {
    let owner_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(owner_id)
    .bind(format!("owner-{owner_id}@example.com"))
    .bind(format!("owner-{owner_id}"))
    .bind("Owner")
    .bind(format!("sub-tables-owner-{owner_id}"))
    .execute(pool)
    .await
    .expect("seed the owner user");
    let organization_id = Uuid::now_v7();
    sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
        .bind(organization_id)
        .bind(name)
        .bind(format!("tables-org-{organization_id}"))
        .bind(owner_id)
        .execute(pool)
        .await
        .expect("seed the organization");
    (organization_id, owner_id)
}

async fn seed_member(
    pool: &PgPool,
    organization_id: Uuid,
    permissions: Option<i64>,
    named: bool,
) -> Caller {
    let user_id = Uuid::now_v7();
    let sub = format!("sub-tables-{user_id}");
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("tables-{user_id}@example.com"))
    .bind(format!("tables-{user_id}"))
    .bind("Member")
    .bind(&sub)
    .execute(pool)
    .await
    .expect("seed the user");
    let member_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name, first_name) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(member_id)
    .bind(organization_id)
    .bind(user_id)
    .bind(if named { "Martin" } else { "Membre" })
    .bind(named.then_some("Léa"))
    .execute(pool)
    .await
    .expect("seed the membership");
    if let Some(permissions) = permissions {
        let role_id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO roles (id, organization_id, name, permissions) VALUES ($1, $2, $3, $4)",
        )
        .bind(role_id)
        .bind(organization_id)
        .bind(format!("role-{role_id}"))
        .bind(permissions)
        .execute(pool)
        .await
        .expect("seed the role");
        sqlx::query("INSERT INTO member_roles (id, member_id, role_id) VALUES ($1, $2, $3)")
            .bind(Uuid::now_v7())
            .bind(member_id)
            .bind(role_id)
            .execute(pool)
            .await
            .expect("assign the role");
    }
    Caller {
        token: issuer::mint(&sub),
        user_id,
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
        "--rate-limit-per-minute".to_owned(),
        "100000".to_owned(),
        "--auth-issuer".to_owned(),
        issuer_url.to_owned(),
        "--file-storage-auto-create-bucket".to_owned(),
        "false".to_owned(),
    ]
}

pub struct Reply {
    pub status: u16,
    pub body: Value,
}

impl App {
    pub fn url(&self, organization_id: Uuid, suffix: &str) -> String {
        format!(
            "{}/api/v1/organizations/{organization_id}/knowledge/tables{suffix}",
            self.base_url
        )
    }

    pub async fn call(
        &self,
        method: reqwest::Method,
        token: &str,
        organization_id: Uuid,
        suffix: &str,
        payload: Option<Value>,
    ) -> Reply {
        let request = reqwest::Client::new()
            .request(method, self.url(organization_id, suffix))
            .bearer_auth(token);
        let request = match payload {
            Some(payload) => request.json(&payload),
            None => request,
        };
        let response = request.send().await.expect("the api answers");
        let status = response.status().as_u16();
        let text = response.text().await.expect("the answer has a body");
        let body = if text.is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&text).unwrap_or(Value::String(text))
        };
        Reply { status, body }
    }

    pub async fn as_manager(
        &self,
        method: reqwest::Method,
        suffix: &str,
        payload: Option<Value>,
    ) -> Reply {
        self.call(
            method,
            &self.manager.token,
            self.organization_id,
            suffix,
            payload,
        )
        .await
    }

    pub async fn seed_customer(&self, organization_id: Uuid, name: &str) -> Uuid {
        let id = Uuid::now_v7();
        sqlx::query("INSERT INTO customers (id, org_id, name) VALUES ($1, $2, $3)")
            .bind(id)
            .bind(organization_id)
            .bind(name)
            .execute(&self.pool)
            .await
            .expect("seed the customer");
        id
    }

    pub async fn seed_file(&self, organization_id: Uuid, title: &str) -> Uuid {
        let id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO knowledge_articles (id, org_id, kind, title, file_key, mime_type, size_bytes)
             VALUES ($1, $2, 'file', $3, $4, 'application/pdf', 1024)",
        )
        .bind(id)
        .bind(organization_id)
        .bind(title)
        .bind(format!("knowledge-test/{id}"))
        .execute(&self.pool)
        .await
        .expect("seed the file");
        id
    }

    pub async fn cleanup(&self) {
        for organization_id in [self.organization_id, self.intruder_organization_id] {
            for statement in [
                "DELETE FROM automation.event WHERE org_id = $1",
                "DELETE FROM knowledge_tables WHERE org_id = $1",
                "DELETE FROM knowledge_articles WHERE org_id = $1",
                "DELETE FROM customers WHERE org_id = $1",
                "DELETE FROM member_roles WHERE member_id IN (SELECT id FROM organization_members WHERE organization_id = $1)",
                "DELETE FROM roles WHERE organization_id = $1",
                "DELETE FROM organization_members WHERE organization_id = $1",
                "DELETE FROM organizations WHERE id = $1",
            ] {
                sqlx::query(statement)
                    .bind(organization_id)
                    .execute(&self.pool)
                    .await
                    .unwrap_or_else(|e| panic!("cleanup failed on `{statement}`: {e}"));
            }
        }
        sqlx::query("DELETE FROM users WHERE id = ANY($1)")
            .bind(&self.users)
            .execute(&self.pool)
            .await
            .expect("clear the fixture users");
    }
}

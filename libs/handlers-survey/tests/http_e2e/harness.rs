use std::{net::SocketAddr, sync::Arc};

use args::Args;
use clap::Parser;
use reqwest::{Method, StatusCode};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::issuer;

const VIEW_SURVEYS: i64 = 1 << 25;
const MANAGE_SURVEYS: i64 = 1 << 26;

pub const QUESTION: &str = "0195f0a0-0000-7000-8000-000000000001";
pub const END: &str = "0195f0a0-0000-7000-8000-000000000002";

pub struct App {
    base_url: String,
    client: reqwest::Client,
    pool: PgPool,
    pub organization_id: Uuid,
    pub other_organization_id: Uuid,
    pub flag_off_organization_id: Uuid,
    pub manager: String,
    pub viewer: String,
    pub member: String,
    pub other_manager: String,
    pub other_member: String,
    pub flag_off_manager: String,
    users: Vec<Uuid>,
}

pub async fn start() -> App {
    let issuer_url = issuer::spawn();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set to run the survey end-to-end tests");
    let redis_url = std::env::var("RATE_LIMIT_REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_owned());

    let pool = PgPool::connect(&database_url)
        .await
        .expect("connect to the test database");

    let mut users = Vec::new();
    let manager_id = seed_user(&pool, &mut users).await;
    let viewer_id = seed_user(&pool, &mut users).await;
    let member_id = seed_user(&pool, &mut users).await;
    let other_manager_id = seed_user(&pool, &mut users).await;
    let other_member_id = seed_user(&pool, &mut users).await;
    let flag_off_manager_id = seed_user(&pool, &mut users).await;

    let organization_id = seed_organization(&pool, manager_id, true).await;
    seed_seat(
        &pool,
        organization_id,
        manager_id,
        Some(MANAGE_SURVEYS | VIEW_SURVEYS),
    )
    .await;
    seed_seat(&pool, organization_id, viewer_id, Some(VIEW_SURVEYS)).await;
    seed_seat(&pool, organization_id, member_id, None).await;

    let other_organization_id = seed_organization(&pool, other_manager_id, true).await;
    seed_seat(
        &pool,
        other_organization_id,
        other_manager_id,
        Some(MANAGE_SURVEYS | VIEW_SURVEYS),
    )
    .await;
    seed_seat(&pool, other_organization_id, other_member_id, None).await;

    let flag_off_organization_id = seed_organization(&pool, flag_off_manager_id, false).await;
    seed_seat(
        &pool,
        flag_off_organization_id,
        flag_off_manager_id,
        Some(MANAGE_SURVEYS | VIEW_SURVEYS),
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
    let router = handlers_survey::router(&state).with_state(state);
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
        client: reqwest::Client::new(),
        pool,
        organization_id,
        other_organization_id,
        flag_off_organization_id,
        manager: issuer::mint(&sub_of(manager_id)),
        viewer: issuer::mint(&sub_of(viewer_id)),
        member: issuer::mint(&sub_of(member_id)),
        other_manager: issuer::mint(&sub_of(other_manager_id)),
        other_member: issuer::mint(&sub_of(other_member_id)),
        flag_off_manager: issuer::mint(&sub_of(flag_off_manager_id)),
        users,
    }
}

impl App {
    pub async fn call(
        &self,
        method: Method,
        path: &str,
        token: Option<&str>,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let mut request = self
            .client
            .request(method, format!("{}/api/v1{path}", self.base_url));
        if let Some(token) = token {
            request = request.bearer_auth(token);
        }
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await.expect("the api answers");
        let status = response.status();
        let text = response.text().await.expect("the api sends a body");
        let json = if text.is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&text).unwrap_or(Value::String(text))
        };
        (status, json)
    }

    pub fn org(&self, suffix: &str) -> String {
        format!("/organizations/{}{suffix}", self.organization_id)
    }

    pub fn other_org(&self, suffix: &str) -> String {
        format!("/organizations/{}{suffix}", self.other_organization_id)
    }

    pub fn flag_off_org(&self, suffix: &str) -> String {
        format!("/organizations/{}{suffix}", self.flag_off_organization_id)
    }

    pub async fn opened_survey(&self, path: impl Fn(&str) -> String, token: &str) -> String {
        self.opened_survey_as(path, token, "MEMBERS").await
    }

    pub async fn set_surveys_flag(&self, organization_id: Uuid, enabled: bool) {
        let statement = if enabled {
            "INSERT INTO organization_feature_flags (org_id, flag) VALUES ($1, 'surveys') ON CONFLICT DO NOTHING"
        } else {
            "DELETE FROM organization_feature_flags WHERE org_id = $1 AND flag = 'surveys'"
        };
        sqlx::query(statement)
            .bind(organization_id)
            .execute(&self.pool)
            .await
            .expect("toggle the surveys flag");
    }

    pub async fn opened_survey_as(
        &self,
        path: impl Fn(&str) -> String,
        token: &str,
        visibility: &str,
    ) -> String {
        let id = self.draft_survey(&path, token, visibility).await;
        let (status, body) = self
            .call(
                Method::POST,
                &path(&format!("/surveys/{id}/open")),
                Some(token),
                None,
            )
            .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        id
    }

    pub async fn draft_survey(
        &self,
        path: impl Fn(&str) -> String,
        token: &str,
        visibility: &str,
    ) -> String {
        let (status, body) = self
            .call(
                Method::POST,
                &path("/surveys"),
                Some(token),
                Some(serde_json::json!({"title": "Satisfaction", "visibility": visibility})),
            )
            .await;
        assert_eq!(status, StatusCode::CREATED, "{body}");
        let id = body["data"]["id"]
            .as_str()
            .expect("the survey has an id")
            .to_owned();
        let (status, body) = self
            .call(
                Method::PATCH,
                &path(&format!("/surveys/{id}")),
                Some(token),
                Some(serde_json::json!({"graph": graph()})),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        id
    }

    pub async fn cleanup(&self) {
        for organization in [
            self.organization_id,
            self.other_organization_id,
            self.flag_off_organization_id,
        ] {
            for statement in [
                "DELETE FROM automation.event WHERE org_id = $1",
                "DELETE FROM survey.response WHERE org_id = $1",
                "DELETE FROM survey.survey WHERE org_id = $1",
                "DELETE FROM survey.campaign WHERE org_id = $1",
                "DELETE FROM member_roles WHERE role_id IN (SELECT id FROM roles WHERE organization_id = $1)",
                "DELETE FROM roles WHERE organization_id = $1",
                "DELETE FROM organization_members WHERE organization_id = $1",
                "DELETE FROM organizations WHERE id = $1",
            ] {
                sqlx::query(statement)
                    .bind(organization)
                    .execute(&self.pool)
                    .await
                    .unwrap_or_else(|e| panic!("cleanup failed on `{statement}`: {e}"));
            }
        }
        for user in &self.users {
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user)
                .execute(&self.pool)
                .await
                .expect("clear a fixture user");
        }
    }
}

pub fn graph() -> Value {
    serde_json::json!({
        "nodes": [
            {"type": "QUESTION", "id": QUESTION, "kind": "SINGLE_CHOICE", "label": "Le stage vous a-t-il plu ?", "required": true, "options": ["Oui", "Non"]},
            {"type": "END", "id": END}
        ],
        "edges": [{"from": QUESTION, "to": END, "branch": "NEXT"}],
        "start": QUESTION
    })
}

fn sub_of(user_id: Uuid) -> String {
    format!("sub-survey-http-{user_id}")
}

async fn seed_user(pool: &PgPool, users: &mut Vec<Uuid>) -> Uuid {
    let user_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("survey-{user_id}@example.com"))
    .bind(format!("survey-{user_id}"))
    .bind("Survey Http")
    .bind(sub_of(user_id))
    .execute(pool)
    .await
    .expect("seed the user");
    users.push(user_id);
    user_id
}

async fn seed_organization(pool: &PgPool, owner: Uuid, surveys: bool) -> Uuid {
    let organization_id = Uuid::now_v7();
    sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
        .bind(organization_id)
        .bind("Survey Org")
        .bind(format!("survey-http-{organization_id}"))
        .bind(owner)
        .execute(pool)
        .await
        .expect("seed the organization");
    if surveys {
        sqlx::query("INSERT INTO organization_feature_flags (org_id, flag) VALUES ($1, 'surveys')")
            .bind(organization_id)
            .execute(pool)
            .await
            .expect("enable surveys");
    }
    organization_id
}

async fn seed_seat(pool: &PgPool, organization_id: Uuid, user_id: Uuid, permissions: Option<i64>) {
    let member_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name) VALUES ($1, $2, $3, $4)",
    )
    .bind(member_id)
    .bind(organization_id)
    .bind(user_id)
    .bind("Seat")
    .execute(pool)
    .await
    .expect("seed the membership");
    let Some(permissions) = permissions else {
        return;
    };
    let role_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO roles (id, organization_id, name, permissions) VALUES ($1, $2, $3, $4)",
    )
    .bind(role_id)
    .bind(organization_id)
    .bind(format!("Survey {permissions}"))
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
        "--auth-audience".to_owned(),
        "mestier-api".to_owned(),
        "--auth-client-id".to_owned(),
        "mestier".to_owned(),
        "--auth-client-secret".to_owned(),
        "mestier".to_owned(),
        "--auth-trusted-clients".to_owned(),
        "mestier".to_owned(),
        "--file-storage-auto-create-bucket".to_owned(),
        "false".to_owned(),
    ]
}

use std::{net::SocketAddr, sync::Arc};

use args::Args;
use authz::Subject;
use clap::Parser;
use handlers::AppState;
use mestier_core::{
    CreateTicketCommand, CustomFields, OrganizationId, Permissions, Ticket, TicketStatus,
    TransitionTicketCommand,
};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::issuer;

pub struct Caller {
    pub token: String,
    pub member_id: Uuid,
    user_id: Uuid,
}

pub struct App {
    pub base_url: String,
    pub state: AppState,
    pub pool: PgPool,
    pub organization_id: Uuid,
    pub other_organization_id: Uuid,
    pub other_member_id: Uuid,
    pub admin: Caller,
    pub viewer: Caller,
    pub outsider: Caller,
    other_owner_user_id: Uuid,
}

pub async fn start() -> App {
    let issuer_url = issuer::spawn();
    let database_url = std::env::var("DATABASE_URL").unwrap();
    let redis_url = std::env::var("RATE_LIMIT_REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_owned());

    let pool = PgPool::connect(&database_url).await.unwrap();
    let fixture = seed(&pool).await;

    let args = Arc::new(Args::parse_from(args_for(
        &database_url,
        &redis_url,
        &issuer_url,
    )));
    let state = handlers::state(args).await.unwrap();

    let router = handlers_support::router(&state).with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await;
    });

    App {
        base_url: format!("http://{addr}"),
        state,
        pool,
        organization_id: fixture.organization_id,
        other_organization_id: fixture.other_organization_id,
        other_member_id: fixture.other_member_id,
        admin: fixture.admin,
        viewer: fixture.viewer,
        outsider: fixture.outsider,
        other_owner_user_id: fixture.other_owner_user_id,
    }
}

impl App {
    pub fn tickets_url(&self, suffix: &str) -> String {
        format!(
            "{}/api/v1/organizations/{}/support/tickets{suffix}",
            self.base_url, self.organization_id
        )
    }

    pub fn ticket_url(&self, ticket_id: Uuid, suffix: &str) -> String {
        self.tickets_url(&format!("/{ticket_id}{suffix}"))
    }

    pub async fn seed_ticket(
        &self,
        organization_id: Uuid,
        subject: &str,
        custom_fields: Value,
    ) -> Ticket {
        self.state
            .usecase
            .create_support_ticket(CreateTicketCommand {
                actor: Subject::system(),
                organization_id: OrganizationId(organization_id),
                subject: subject.to_owned(),
                body: format!("{subject} body"),
                custom_fields: CustomFields::try_from(custom_fields).unwrap(),
                external_id: None,
            })
            .await
            .unwrap()
            .ticket
    }

    pub async fn set_status(&self, ticket: &Ticket, status: TicketStatus) {
        self.state
            .usecase
            .transition_support_ticket(TransitionTicketCommand {
                actor: Subject::system(),
                organization_id: ticket.organization_id,
                ticket_id: ticket.id,
                status,
            })
            .await
            .unwrap();
    }

    pub async fn cleanup(&self) {
        let organizations = [self.organization_id, self.other_organization_id];

        for statement in [
            "DELETE FROM automation.event WHERE org_id = ANY($1)",
            "DELETE FROM support_tickets WHERE org_id = ANY($1)",
            "DELETE FROM support_ticket_counters WHERE org_id = ANY($1)",
            "DELETE FROM organization_members WHERE organization_id = ANY($1)",
            "DELETE FROM organizations WHERE id = ANY($1)",
        ] {
            sqlx::query(statement)
                .bind(&organizations[..])
                .execute(&self.pool)
                .await
                .unwrap();
        }

        for user_id in [
            self.admin.user_id,
            self.viewer.user_id,
            self.outsider.user_id,
            self.other_owner_user_id,
        ] {
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&self.pool)
                .await
                .unwrap();
        }
    }
}

struct Fixture {
    organization_id: Uuid,
    other_organization_id: Uuid,
    other_member_id: Uuid,
    other_owner_user_id: Uuid,
    admin: Caller,
    viewer: Caller,
    outsider: Caller,
}

async fn seed(pool: &PgPool) -> Fixture {
    let (admin_user_id, admin_sub) = seed_user(pool, "admin").await;
    let (viewer_user_id, viewer_sub) = seed_user(pool, "viewer").await;
    let (outsider_user_id, outsider_sub) = seed_user(pool, "outsider").await;
    let (other_owner_user_id, _) = seed_user(pool, "other-owner").await;

    let organization_id = seed_organization(pool, admin_user_id).await;
    let other_organization_id = seed_organization(pool, other_owner_user_id).await;

    let admin_member_id = seed_member(pool, organization_id, admin_user_id, "Admin").await;
    let viewer_member_id = seed_member(pool, organization_id, viewer_user_id, "Viewer").await;
    let outsider_member_id = seed_member(pool, organization_id, outsider_user_id, "Outsider").await;
    let other_member_id =
        seed_member(pool, other_organization_id, other_owner_user_id, "Other").await;

    let admin_role_id = seed_role(
        pool,
        organization_id,
        "support-admin",
        (Permissions::VIEW_SUPPORT | Permissions::MANAGE_SUPPORT).0,
    )
    .await;
    assign_role(pool, admin_member_id, admin_role_id).await;

    let viewer_role_id = seed_role(
        pool,
        organization_id,
        "support-viewer",
        Permissions::VIEW_SUPPORT.0,
    )
    .await;
    assign_role(pool, viewer_member_id, viewer_role_id).await;

    Fixture {
        organization_id,
        other_organization_id,
        other_member_id,
        other_owner_user_id,
        admin: Caller {
            token: issuer::mint(&admin_sub),
            member_id: admin_member_id,
            user_id: admin_user_id,
        },
        viewer: Caller {
            token: issuer::mint(&viewer_sub),
            member_id: viewer_member_id,
            user_id: viewer_user_id,
        },
        outsider: Caller {
            token: issuer::mint(&outsider_sub),
            member_id: outsider_member_id,
            user_id: outsider_user_id,
        },
    }
}

async fn seed_user(pool: &PgPool, label: &str) -> (Uuid, String) {
    let user_id = Uuid::now_v7();
    let sub = format!("sub-support-{label}-{user_id}");
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("{label}-{user_id}@example.com"))
    .bind(format!("{label}-{user_id}"))
    .bind(label)
    .bind(&sub)
    .execute(pool)
    .await
    .unwrap();

    (user_id, sub)
}

async fn seed_organization(pool: &PgPool, owner_id: Uuid) -> Uuid {
    let organization_id = Uuid::now_v7();
    sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
        .bind(organization_id)
        .bind("Support Org")
        .bind(format!("support-org-{organization_id}"))
        .bind(owner_id)
        .execute(pool)
        .await
        .unwrap();

    organization_id
}

async fn seed_member(pool: &PgPool, organization_id: Uuid, user_id: Uuid, last_name: &str) -> Uuid {
    let member_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name) VALUES ($1, $2, $3, $4)",
    )
    .bind(member_id)
    .bind(organization_id)
    .bind(user_id)
    .bind(last_name)
    .execute(pool)
    .await
    .unwrap();

    member_id
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
    .unwrap();

    role_id
}

async fn assign_role(pool: &PgPool, member_id: Uuid, role_id: Uuid) {
    sqlx::query("INSERT INTO member_roles (id, member_id, role_id) VALUES ($1, $2, $3)")
        .bind(Uuid::now_v7())
        .bind(member_id)
        .bind(role_id)
        .execute(pool)
        .await
        .unwrap();
}

fn args_for(database_url: &str, redis_url: &str, issuer_url: &str) -> Vec<String> {
    let db = url::Url::parse(database_url).unwrap();

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

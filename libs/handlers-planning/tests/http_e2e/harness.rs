//! Brings the planning API up on a real socket, against a real database —
//! scoped to what `assignment_report`'s own tests need. The rest of this
//! crate's routes (tasks, projects, labels, work time, planning views) have
//! no end-to-end coverage yet; this harness does not attempt to backfill it,
//! only to cover the manager's half of the correction loop this workstream
//! adds.
//!
//! `AppState` comes from `handlers::state`, the same function the binary
//! calls, so the test exercises the production wiring. Only the auth issuer
//! is pointed elsewhere, at a local JWKS server — mirrors
//! `libs/handlers-field/tests/http_e2e/harness.rs`.

use std::{net::SocketAddr, sync::Arc};

use args::Args;
use clap::Parser;
use sqlx::PgPool;
use uuid::Uuid;

use crate::issuer;

pub struct App {
    pub base_url: String,
    /// The manager's token — the organization's owner, so every route this
    /// suite calls passes membership.
    pub token: String,
    pub pool: PgPool,
    pub organization_id: Uuid,
    pub task_assignment_id: Uuid,
    pub assignee_member_id: Uuid,
    manager_user_id: Uuid,
    assignee_user_id: Uuid,
}

pub async fn start() -> App {
    let issuer_url = issuer::spawn();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set to run the planning end-to-end tests");
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

    let router = handlers_planning::router(&state).with_state(state);
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
        token: issuer::mint(&fixture.manager_sub),
        pool,
        organization_id: fixture.organization_id,
        task_assignment_id: fixture.task_assignment_id,
        assignee_member_id: fixture.assignee_member_id,
        manager_user_id: fixture.manager_user_id,
        assignee_user_id: fixture.assignee_user_id,
    }
}

impl App {
    pub fn reports_url(&self, suffix: &str) -> String {
        format!(
            "{}/api/v1/organizations/{}/assignment-reports{suffix}",
            self.base_url, self.organization_id
        )
    }

    pub fn resolution_url(&self, assignment_report_id: &str) -> String {
        format!(
            "{}/api/v1/assignment-reports/{assignment_report_id}/resolution",
            self.base_url
        )
    }

    pub fn recurrences_url(&self, suffix: &str) -> String {
        format!(
            "{}/api/v1/organizations/{}/task-recurrences{suffix}",
            self.base_url, self.organization_id
        )
    }

    pub fn project_templates_url(&self, suffix: &str) -> String {
        format!(
            "{}/api/v1/organizations/{}/project-templates{suffix}",
            self.base_url, self.organization_id
        )
    }

    pub fn recurrence_url(&self, task_recurrence_id: &str) -> String {
        format!(
            "{}/api/v1/task-recurrences/{task_recurrence_id}",
            self.base_url
        )
    }

    pub fn task_url(&self, task_id: &str) -> String {
        format!(
            "{}/api/v1/organizations/{}/tasks/{task_id}",
            self.base_url, self.organization_id
        )
    }

    /// The materialized occurrences of `recurrence_id`, oldest first — reads
    /// straight from the database rather than through the API, since the
    /// suite needs an occurrence's own id before it can address it by URL.
    pub async fn occurrence_task_ids(&self, recurrence_id: &str) -> Vec<Uuid> {
        let recurrence_id: Uuid = recurrence_id.parse().expect("a valid recurrence id");
        sqlx::query_scalar(
            "SELECT id FROM tasks WHERE recurrence_id = $1 AND deleted_at IS NULL \
             ORDER BY occurrence_date ASC",
        )
        .bind(recurrence_id)
        .fetch_all(&self.pool)
        .await
        .expect("read the materialized occurrences")
    }

    /// A pending report on the fixture's own assignment, seeded directly:
    /// filing one is `handlers-field`'s own route, covered by its suite —
    /// this harness only needs a row to arbitrate.
    pub async fn seed_pending_report(&self, reported_minutes: i32) -> Uuid {
        let id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO assignment_reports (id, org_id, task_assignment_id, reported_minutes, reported_by)
             VALUES ($1, $2, $3, $4, (SELECT member_id FROM task_assignments WHERE id = $3))",
        )
        .bind(id)
        .bind(self.organization_id)
        .bind(self.task_assignment_id)
        .bind(reported_minutes)
        .execute(&self.pool)
        .await
        .expect("seed the pending report");

        id
    }

    /// Removes the fixture, child rows first — explicit and ordered rather
    /// than relying on cascades, mirroring the field harness's own note.
    pub async fn cleanup(&self) {
        for statement in [
            "DELETE FROM automation.event WHERE org_id = $1",
            "DELETE FROM assignment_reports WHERE org_id = $1",
            "DELETE FROM task_assignments WHERE org_id = $1",
            "DELETE FROM tasks WHERE org_id = $1",
            "DELETE FROM task_recurrences WHERE org_id = $1",
            "DELETE FROM projects WHERE org_id = $1",
            // Cascades to `project_template_tasks`.
            "DELETE FROM project_templates WHERE org_id = $1",
            "DELETE FROM organization_members WHERE organization_id = $1",
            "DELETE FROM organizations WHERE id = $1",
        ] {
            sqlx::query(statement)
                .bind(self.organization_id)
                .execute(&self.pool)
                .await
                .unwrap_or_else(|e| panic!("cleanup failed on `{statement}`: {e}"));
        }

        for user_id in [self.manager_user_id, self.assignee_user_id] {
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&self.pool)
                .await
                .expect("clear the fixture user");
        }
    }
}

struct Fixture {
    manager_sub: String,
    manager_user_id: Uuid,
    assignee_user_id: Uuid,
    organization_id: Uuid,
    task_assignment_id: Uuid,
    assignee_member_id: Uuid,
}

/// An organization, its owner (the manager, the caller for every test), a
/// second member (the assignee), a task, and its assignment to that member.
async fn seed(pool: &PgPool) -> Fixture {
    let manager_user_id = Uuid::now_v7();
    let manager_sub = format!("sub-planning-manager-{manager_user_id}");
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(manager_user_id)
    .bind(format!("manager-{manager_user_id}@example.com"))
    .bind(format!("manager-{manager_user_id}"))
    .bind("Manager")
    .bind(&manager_sub)
    .execute(pool)
    .await
    .expect("seed the manager user");

    let organization_id = Uuid::now_v7();
    sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
        .bind(organization_id)
        .bind("Test Org")
        .bind(format!("test-org-{organization_id}"))
        .bind(manager_user_id)
        .execute(pool)
        .await
        .expect("seed the organization");

    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name) VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::now_v7())
    .bind(organization_id)
    .bind(manager_user_id)
    .bind("Manager")
    .execute(pool)
    .await
    .expect("seed the manager's own membership");

    let assignee_user_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(assignee_user_id)
    .bind(format!("assignee-{assignee_user_id}@example.com"))
    .bind(format!("assignee-{assignee_user_id}"))
    .bind("Assignee")
    .bind(format!("sub-planning-assignee-{assignee_user_id}"))
    .execute(pool)
    .await
    .expect("seed the assignee user");

    let assignee_member_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name) VALUES ($1, $2, $3, $4)",
    )
    .bind(assignee_member_id)
    .bind(organization_id)
    .bind(assignee_user_id)
    .bind("Assignee")
    .execute(pool)
    .await
    .expect("seed the assignee's own membership");

    let task_id = Uuid::now_v7();
    let now = chrono::Utc::now();
    sqlx::query(
        "INSERT INTO tasks (id, org_id, starts_at, ends_at, all_day, status, title)
         VALUES ($1, $2, $3, $4, false, CAST($5 AS text)::task_status, $6)",
    )
    .bind(task_id)
    .bind(organization_id)
    .bind(now - chrono::Duration::hours(1))
    .bind(now + chrono::Duration::hours(4))
    .bind("PLANNED")
    .bind("Réunion de chantier")
    .execute(pool)
    .await
    .expect("seed the task");

    let task_assignment_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO task_assignments (id, org_id, task_id, member_id) VALUES ($1, $2, $3, $4)",
    )
    .bind(task_assignment_id)
    .bind(organization_id)
    .bind(task_id)
    .bind(assignee_member_id)
    .execute(pool)
    .await
    .expect("seed the assignment");

    Fixture {
        manager_sub,
        manager_user_id,
        assignee_user_id,
        organization_id,
        task_assignment_id,
        assignee_member_id,
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
        "--auth-issuer".to_owned(),
        issuer_url.to_owned(),
        "--file-storage-auto-create-bucket".to_owned(),
        "false".to_owned(),
    ]
}

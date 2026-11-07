use chrono::{Duration, Utc};
use common::{OrganizationId, generate_uuid_v7};
use serde_json::{Map, Value, json};
use sqlx::PgPool;
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::application::test_support::scratch_pool;
use crate::application::{MestierUseCase, default_authorizer};
use crate::domain::automation::connector::connector_catalogue;
use crate::domain::automation::run::{ConnectorInput, ConnectorOutcome, DryRunInput};
use crate::domain::task::commands::CreateTaskCommand;
use crate::infrastructure::automation::connectors::ConnectorRegistry;
use crate::infrastructure::automation::connectors::test_support::assert_same_shape;
use crate::infrastructure::realtime::EventHub;

static SCRATCH: OnceCell<String> = OnceCell::const_new();

const NOT_FOUND: &str = "not found: the targeted object does not exist in this organization";

struct World {
    usecase: MestierUseCase,
    pool: PgPool,
    registry: ConnectorRegistry,
    organization_id: OrganizationId,
}

impl World {
    async fn new(pool: &PgPool) -> Self {
        let owner = generate_uuid_v7();
        sqlx::query(
            "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(owner)
        .bind(format!("owner-{owner}@example.com"))
        .bind(format!("owner-{owner}"))
        .bind("Owner")
        .bind(format!("sub-{owner}"))
        .execute(pool)
        .await
        .unwrap();
        let organization_id = OrganizationId(generate_uuid_v7());
        sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
            .bind(organization_id.0)
            .bind("Treyve Paysages")
            .bind(format!("org-{}", organization_id.0))
            .bind(owner)
            .execute(pool)
            .await
            .unwrap();
        let usecase = MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new());
        Self {
            pool: pool.clone(),
            registry: ConnectorRegistry::new(usecase.clone()),
            usecase,
            organization_id,
        }
    }

    async fn run(&self, kind: &str, pairs: &[(&str, Value)]) -> ConnectorOutcome {
        self.run_as(self.organization_id, kind, pairs).await
    }

    async fn run_as(
        &self,
        organization_id: OrganizationId,
        kind: &str,
        pairs: &[(&str, Value)],
    ) -> ConnectorOutcome {
        let config: Map<String, Value> = pairs
            .iter()
            .map(|(name, value)| ((*name).to_owned(), value.clone()))
            .collect();
        self.registry
            .execute(
                kind,
                1,
                ConnectorInput {
                    org_id: organization_id,
                    run_id: Uuid::from_u128(2),
                    config: &config,
                    credential_id: None,
                    operation: None,
                },
            )
            .await
            .unwrap()
    }

    async fn ok(&self, kind: &str, pairs: &[(&str, Value)]) -> Value {
        match self.run(kind, pairs).await {
            ConnectorOutcome::Produced(value) => value,
            other => panic!("{kind}: expected an output, got {other:?}"),
        }
    }
}

struct Staff {
    member_id: Uuid,
    employee_id: Uuid,
    task_id: Uuid,
    assignment_id: Uuid,
}

impl World {
    async fn staff(&self) -> Staff {
        let member_id = generate_uuid_v7();
        sqlx::query(
            "INSERT INTO organization_members (id, organization_id, last_name) VALUES ($1, $2, $3)",
        )
        .bind(member_id)
        .bind(self.organization_id.0)
        .bind("Terrain")
        .execute(&self.pool)
        .await
        .unwrap();
        let employee_id = generate_uuid_v7();
        sqlx::query(
            "INSERT INTO employees (id, org_id, member_id, hourly_rate_cents, weekly_contract_minutes) \
             VALUES ($1, $2, $3, 3500, 2100)",
        )
        .bind(employee_id)
        .bind(self.organization_id.0)
        .bind(member_id)
        .execute(&self.pool)
        .await
        .unwrap();
        let now = Utc::now();
        let task = self
            .usecase
            .create_task(CreateTaskCommand {
                actor: authz::Subject::system(),
                organization_id: self.organization_id,
                parent_task_id: None,
                title: "Tonte".to_owned(),
                description: None,
                starts_at: Some(now),
                ends_at: Some(now + Duration::hours(2)),
                all_day: false,
                status: None,
                blocks_availability: true,
                customer_id: None,
                customer_context_id: None,
                quote_id: None,
                project_id: None,
                expenses_cents: 0,
                expenses_label: None,
            })
            .await
            .unwrap();
        let assignment_id = generate_uuid_v7();
        sqlx::query(
            "INSERT INTO task_assignments (id, org_id, task_id, member_id) VALUES ($1, $2, $3, $4)",
        )
        .bind(assignment_id)
        .bind(self.organization_id.0)
        .bind(task.id.0)
        .bind(member_id)
        .execute(&self.pool)
        .await
        .unwrap();
        Staff {
            member_id,
            employee_id,
            task_id: task.id.0,
            assignment_id,
        }
    }
}

fn failed(outcome: ConnectorOutcome) -> String {
    match outcome {
        ConnectorOutcome::Failed { error } => error,
        other => panic!("expected a failure, got {other:?}"),
    }
}

async fn pool() -> PgPool {
    scratch_pool("mestier_lot9b_terrain_", &SCRATCH).await
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_member_lists_tasks_works_a_time_entry_and_ends_the_day() {
    let world = World::new(&pool().await).await;
    let staff = world.staff().await;
    let member = json!(staff.member_id.to_string());
    let task = json!(staff.task_id.to_string());

    let tasks = world
        .ok(
            "mestier.terrain.list_tasks",
            &[("member_id", member.clone())],
        )
        .await;
    let started = world
        .ok(
            "mestier.terrain.start_time_entry",
            &[("member_id", member.clone()), ("task_id", task)],
        )
        .await;
    let current = world
        .ok(
            "mestier.terrain.get_current",
            &[("member_id", member.clone())],
        )
        .await;
    let stopped = world
        .ok(
            "mestier.terrain.stop_time_entry",
            &[("time_entry_id", started["id"].clone())],
        )
        .await;
    let photo = world
        .ok(
            "mestier.terrain.attach_time_entry_photo",
            &[
                ("time_entry_id", started["id"].clone()),
                ("phase", json!("AFTER")),
                ("storage_key", json!("photos/a.jpg")),
            ],
        )
        .await;
    let day = world
        .ok("mestier.terrain.end_day", &[("member_id", member)])
        .await;

    assert_eq!(tasks.as_array().unwrap().len(), 1);
    assert_eq!(
        tasks[0]["task_assignment_id"],
        json!(staff.assignment_id.to_string())
    );
    assert_eq!(started["employee_id"], json!(staff.employee_id.to_string()));
    assert_eq!(current["running"]["id"], started["id"]);
    assert!(stopped["ended_at"].is_string());
    assert_eq!(photo["phase"], "AFTER");
    assert!(day["work_date"].is_string());
    assert_same_shape(&tasks, "mestier.terrain.list_tasks");
    assert_same_shape(&started, "mestier.terrain.start_time_entry");
    assert_same_shape(&current, "mestier.terrain.get_current");
    assert_same_shape(&stopped, "mestier.terrain.stop_time_entry");
    assert_same_shape(&photo, "mestier.terrain.attach_time_entry_photo");
    assert_same_shape(&day, "mestier.terrain.end_day");
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_time_report_is_made_amended_listed_and_withdrawn() {
    let world = World::new(&pool().await).await;
    let staff = world.staff().await;
    let member = json!(staff.member_id.to_string());

    let report = world
        .ok(
            "mestier.terrain.report_assignment",
            &[
                ("member_id", member.clone()),
                ("task_assignment_id", json!(staff.assignment_id.to_string())),
                ("reported_minutes", json!(90)),
            ],
        )
        .await;
    let mine = world
        .ok(
            "mestier.terrain.list_assignment_reports",
            &[("member_id", member), ("resolution", json!("PENDING"))],
        )
        .await;
    let amended = world
        .ok(
            "mestier.terrain.amend_assignment_report",
            &[
                ("assignment_report_id", report["id"].clone()),
                ("reported_minutes", json!(120)),
                ("comment", json!("Plus long")),
            ],
        )
        .await;
    let all = world
        .ok("mestier.terrain.list_assignment_reports", &[])
        .await;
    let withdrawn = world
        .ok(
            "mestier.terrain.withdraw_assignment_report",
            &[("assignment_report_id", report["id"].clone())],
        )
        .await;

    assert_eq!(amended["reported_minutes"], 120);
    assert_eq!(mine.as_array().unwrap().len(), 1);
    assert_eq!(all.as_array().unwrap().len(), 1);
    assert_eq!(withdrawn["withdrawn"], json!(true));
    assert_same_shape(&report, "mestier.terrain.report_assignment");
    assert_same_shape(&mine, "mestier.terrain.list_assignment_reports");
    assert_same_shape(&withdrawn, "mestier.terrain.withdraw_assignment_report");
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_object_of_another_organization_fails_with_a_named_error_and_nothing_changes() {
    let pool = pool().await;
    let acme = World::new(&pool).await;
    let globex = World::new(&pool).await;
    let staff = acme.staff().await;
    let member = json!(staff.member_id.to_string());
    let task = json!(staff.task_id.to_string());
    let entry = acme
        .ok(
            "mestier.terrain.start_time_entry",
            &[("member_id", member.clone()), ("task_id", task.clone())],
        )
        .await;
    let report = acme
        .ok(
            "mestier.terrain.report_assignment",
            &[
                ("member_id", member.clone()),
                ("task_assignment_id", json!(staff.assignment_id.to_string())),
                ("reported_minutes", json!(30)),
            ],
        )
        .await;

    let attempts = [
        (
            "mestier.terrain.list_tasks",
            vec![("member_id", member.clone())],
        ),
        (
            "mestier.terrain.get_current",
            vec![("member_id", member.clone())],
        ),
        (
            "mestier.terrain.start_time_entry",
            vec![("member_id", member.clone()), ("task_id", task.clone())],
        ),
        (
            "mestier.terrain.stop_time_entry",
            vec![("time_entry_id", entry["id"].clone())],
        ),
        (
            "mestier.terrain.recover_time_entry",
            vec![
                ("time_entry_id", entry["id"].clone()),
                ("ended_at", json!(Utc::now().to_rfc3339())),
            ],
        ),
        (
            "mestier.terrain.amend_assignment_report",
            vec![
                ("assignment_report_id", report["id"].clone()),
                ("reported_minutes", json!(5)),
            ],
        ),
        (
            "mestier.terrain.withdraw_assignment_report",
            vec![("assignment_report_id", report["id"].clone())],
        ),
        (
            "mestier.terrain.propose_maintenance_intervention",
            vec![
                ("member_id", member),
                ("task_id", task),
                ("done_on", json!("2026-01-01")),
            ],
        ),
    ];

    for (kind, pairs) in attempts {
        let outcome = acme.run_as(globex.organization_id, kind, &pairs).await;
        assert_eq!(failed(outcome), NOT_FOUND, "{kind}");
    }

    let current = acme
        .ok(
            "mestier.terrain.get_current",
            &[("member_id", json!(staff.member_id.to_string()))],
        )
        .await;
    assert_eq!(current["running"]["id"], entry["id"]);
    assert_eq!(current["running"]["ended_at"], Value::Null);
    let minutes: i32 =
        sqlx::query_scalar("SELECT reported_minutes FROM assignment_reports WHERE id = $1")
            .bind(report["id"].as_str().unwrap().parse::<Uuid>().unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(minutes, 30);
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_dry_run_of_a_write_action_writes_nothing() {
    let world = World::new(&pool().await).await;
    let staff = world.staff().await;
    let catalogue = connector_catalogue();
    let descriptor = catalogue
        .get("mestier.terrain.start_time_entry", 1)
        .unwrap();
    let config: Map<String, Value> = [
        ("member_id".to_owned(), json!(staff.member_id.to_string())),
        ("task_id".to_owned(), json!(staff.task_id.to_string())),
    ]
    .into_iter()
    .collect();

    let outcome = ConnectorRegistry::dry_run(
        descriptor,
        &DryRunInput {
            config: &config,
            stored: None,
        },
    );

    assert_eq!(
        outcome,
        Some(ConnectorOutcome::Produced(descriptor.sample_output.clone()))
    );
    let current = world
        .ok(
            "mestier.terrain.get_current",
            &[("member_id", json!(staff.member_id.to_string()))],
        )
        .await;
    assert_eq!(current["running"], Value::Null);
}

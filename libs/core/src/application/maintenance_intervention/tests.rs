//! The visit loop against a real database: @spec-ent-7 to @spec-ent-10 and
//! the rules around them. The fixtures here also serve
//! `maintenance_overview::tests`.

use authz::Subject;
use chrono::{Days, NaiveDate, Utc};
use chrono_tz::Europe::Paris;
use common::{CoreError, OrganizationId, UserId, generate_uuid_v7};
use serde_json::json;
use sqlx::PgPool;
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::{
    AssigneeRef, DueStatus, InterventionId, InterventionStatus, IsoWeekNumber, MaintenanceLineId,
    MemberId, PatchTaskCommand, PlanMaintenanceLinesCommand, ProposeInterventionCommand,
    RecordInterventionCommand, RejectInterventionCommand, Task, ValidateInterventionCommand,
    application::{
        MestierUseCase, default_authorizer, policy,
        test_support::{purge, scratch_pool},
    },
    infrastructure::realtime::EventHub,
};

static MAINTENANCE_SCRATCH: OnceCell<String> = OnceCell::const_new();

pub(crate) async fn pool() -> PgPool {
    scratch_pool("mestier_maintenance_test_", &MAINTENANCE_SCRATCH).await
}

pub(crate) fn usecase(pool: &PgPool) -> MestierUseCase {
    MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new())
}

/// `MANAGE_PLANNING`, the bit behind `planning.manage`.
const MANAGE_PLANNING_BITS: i64 = 1 << 8;

pub(crate) struct Person {
    pub user_id: Uuid,
    pub member_id: MemberId,
}

impl Person {
    pub fn subject(&self) -> Subject {
        policy::user_subject(UserId(self.user_id), Vec::new())
    }
}

pub(crate) struct Site {
    pub customer_id: Uuid,
    pub context_id: Uuid,
}

pub(crate) struct Contract {
    pub id: Uuid,
    pub project_id: Uuid,
    pub line_ids: Vec<MaintenanceLineId>,
}

/// One organization with a manager (holding `planning.manage`) and three
/// members who hold nothing: a plain one, Bernard and Paul.
pub(crate) struct World {
    pub pool: PgPool,
    pub usecase: MestierUseCase,
    pub organization_id: OrganizationId,
    pub manager: Person,
    pub plain: Person,
    pub bernard: Person,
    pub paul: Person,
}

async fn seed_person(
    pool: &PgPool,
    organization_id: OrganizationId,
    label: &str,
    role_id: Option<Uuid>,
) -> Person {
    let user_id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("{label}-{user_id}@example.com"))
    .bind(format!("{label}-{user_id}"))
    .bind(label)
    .bind(format!("sub-{user_id}"))
    .execute(pool)
    .await
    .unwrap();
    seed_member(pool, organization_id, user_id, label, role_id).await
}

async fn seed_member(
    pool: &PgPool,
    organization_id: OrganizationId,
    user_id: Uuid,
    label: &str,
    role_id: Option<Uuid>,
) -> Person {
    let member_id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(member_id)
    .bind(organization_id.0)
    .bind(user_id)
    .bind(label)
    .execute(pool)
    .await
    .unwrap();
    if let Some(role_id) = role_id {
        sqlx::query("INSERT INTO member_roles (id, member_id, role_id) VALUES ($1, $2, $3)")
            .bind(generate_uuid_v7())
            .bind(member_id)
            .bind(role_id)
            .execute(pool)
            .await
            .unwrap();
    }
    Person {
        user_id,
        member_id: MemberId(member_id),
    }
}

impl World {
    pub async fn new(pool: &PgPool) -> Self {
        let manager_user = generate_uuid_v7();
        sqlx::query(
            "INSERT INTO users (id, email, username, display_name, sub) \
             VALUES ($1, $2, $3, 'Manager', $4)",
        )
        .bind(manager_user)
        .bind(format!("manager-{manager_user}@example.com"))
        .bind(format!("manager-{manager_user}"))
        .bind(format!("sub-{manager_user}"))
        .execute(pool)
        .await
        .unwrap();

        let organization_id = OrganizationId(generate_uuid_v7());
        sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
            .bind(organization_id.0)
            .bind("Treyve Paysages")
            .bind(format!("org-{}", organization_id.0))
            .bind(manager_user)
            .execute(pool)
            .await
            .unwrap();

        let role_id = generate_uuid_v7();
        sqlx::query(
            "INSERT INTO roles (id, organization_id, name, permissions) VALUES ($1, $2, $3, $4)",
        )
        .bind(role_id)
        .bind(organization_id.0)
        .bind("Conducteur")
        .bind(MANAGE_PLANNING_BITS)
        .execute(pool)
        .await
        .unwrap();

        Self {
            pool: pool.clone(),
            usecase: usecase(pool),
            organization_id,
            manager: seed_member(
                pool,
                organization_id,
                manager_user,
                "Manager",
                Some(role_id),
            )
            .await,
            plain: seed_person(pool, organization_id, "Plain", None).await,
            bernard: seed_person(pool, organization_id, "Bernard", None).await,
            paul: seed_person(pool, organization_id, "Paul", None).await,
        }
    }

    pub async fn seed_site(&self, customer: &str, city: Option<&str>) -> Site {
        let customer_id = generate_uuid_v7();
        sqlx::query("INSERT INTO customers (id, org_id, name) VALUES ($1, $2, $3)")
            .bind(customer_id)
            .bind(self.organization_id.0)
            .bind(customer)
            .execute(&self.pool)
            .await
            .unwrap();
        let context_id = generate_uuid_v7();
        sqlx::query(
            "INSERT INTO customer_contexts (id, customer_id, label, city) VALUES ($1, $2, $3, $4)",
        )
        .bind(context_id)
        .bind(customer_id)
        .bind(format!("Jardin {customer}"))
        .bind(city)
        .execute(&self.pool)
        .await
        .unwrap();
        Site {
            customer_id,
            context_id,
        }
    }

    /// Inserts a contract and its lines straight into the tables, so these
    /// tests do not depend on the contract use cases. `lines` is
    /// `(label, cadence)` with the cadence in its stored JSON form.
    pub async fn seed_contract(
        &self,
        site: &Site,
        season: (i32, i16, i16),
        lines: &[(&str, serde_json::Value)],
    ) -> Contract {
        let project_id = generate_uuid_v7();
        sqlx::query(
            "INSERT INTO projects (id, org_id, name, customer_id, customer_context_id) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(project_id)
        .bind(self.organization_id.0)
        .bind(format!("Entretien {}", season.0))
        .bind(site.customer_id)
        .bind(site.context_id)
        .execute(&self.pool)
        .await
        .unwrap();

        let id = generate_uuid_v7();
        sqlx::query(
            "INSERT INTO maintenance_contracts \
             (id, org_id, customer_id, customer_context_id, project_id, season_year, \
              first_week, last_week) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(id)
        .bind(self.organization_id.0)
        .bind(site.customer_id)
        .bind(site.context_id)
        .bind(project_id)
        .bind(season.0 as i16)
        .bind(season.1)
        .bind(season.2)
        .execute(&self.pool)
        .await
        .unwrap();

        let mut line_ids = Vec::new();
        for (position, (label, cadence)) in lines.iter().enumerate() {
            let line_id = generate_uuid_v7();
            sqlx::query(
                "INSERT INTO maintenance_lines \
                 (id, org_id, contract_id, label, cadence, minutes_per_intervention, position) \
                 VALUES ($1, $2, $3, $4, $5, 90, $6)",
            )
            .bind(line_id)
            .bind(self.organization_id.0)
            .bind(id)
            .bind(*label)
            .bind(cadence)
            .bind(position as i32)
            .execute(&self.pool)
            .await
            .unwrap();
            line_ids.push(MaintenanceLineId(line_id));
        }
        Contract {
            id,
            project_id,
            line_ids,
        }
    }

    pub async fn seed_visit(
        &self,
        line_id: MaintenanceLineId,
        done_on: NaiveDate,
        status: InterventionStatus,
    ) -> InterventionId {
        let id = generate_uuid_v7();
        let resolved = status != InterventionStatus::Proposed;
        sqlx::query(
            "INSERT INTO maintenance_interventions \
             (id, org_id, line_id, done_on, status, proposed_by, resolved_by, resolved_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(id)
        .bind(self.organization_id.0)
        .bind(line_id.0)
        .bind(done_on)
        .bind(status.as_str())
        .bind(self.manager.member_id.0)
        .bind(resolved.then_some(self.manager.member_id.0))
        .bind(resolved.then(Utc::now))
        .execute(&self.pool)
        .await
        .unwrap();
        InterventionId(id)
    }

    /// Plans one line into a backlog task, then assigns it to `assignee`.
    pub async fn assigned_task(&self, line_id: MaintenanceLineId, assignee: &Person) -> Task {
        let planned = self.plan(&[line_id], 2026, 43).await.unwrap();
        let task = planned.into_iter().next().unwrap();
        self.usecase
            .patch_task(PatchTaskCommand {
                assignees: Some(vec![AssigneeRef(assignee.member_id)]),
                ..PatchTaskCommand::new(task.id, Subject::system())
            })
            .await
            .unwrap()
    }

    pub async fn plan(
        &self,
        line_ids: &[MaintenanceLineId],
        year: i32,
        week: u8,
    ) -> Result<Vec<Task>, CoreError> {
        self.usecase
            .plan_maintenance_lines(PlanMaintenanceLinesCommand {
                actor: self.manager.subject(),
                organization_id: self.organization_id,
                line_ids: line_ids.to_vec(),
                year,
                week: IsoWeekNumber::try_from(week).unwrap(),
            })
            .await
    }

    pub async fn grid(&self, year: i32) -> crate::Grid {
        self.usecase
            .maintenance_grid(self.manager.subject(), self.organization_id, year)
            .await
            .unwrap()
    }

    pub async fn recap(&self, year: i32, week: u8) -> crate::Recap {
        self.usecase
            .maintenance_recap(
                self.manager.subject(),
                self.organization_id,
                year,
                IsoWeekNumber::try_from(week).unwrap(),
            )
            .await
            .unwrap()
    }

    pub async fn event_names(&self) -> Vec<String> {
        sqlx::query_scalar(
            "SELECT name FROM automation.event \
             WHERE org_id = $1 AND name LIKE 'maintenance_intervention.%' \
             ORDER BY occurred_at, id",
        )
        .bind(self.organization_id.0)
        .fetch_all(&self.pool)
        .await
        .unwrap()
    }

    pub async fn count_interventions(&self) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM maintenance_interventions WHERE org_id = $1")
            .bind(self.organization_id.0)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    pub async fn cleanup(self) {
        for statement in [
            "DELETE FROM automation.event WHERE org_id = $1",
            "DELETE FROM maintenance_interventions WHERE org_id = $1",
            "DELETE FROM task_assignments WHERE org_id = $1",
            "DELETE FROM tasks WHERE org_id = $1",
            "DELETE FROM maintenance_lines WHERE org_id = $1",
            "DELETE FROM maintenance_contracts WHERE org_id = $1",
            "DELETE FROM projects WHERE org_id = $1",
            "DELETE FROM customer_contexts WHERE customer_id IN (SELECT id FROM customers WHERE org_id = $1)",
            "DELETE FROM customers WHERE org_id = $1",
            "DELETE FROM member_roles WHERE member_id IN (SELECT id FROM organization_members WHERE organization_id = $1)",
            "DELETE FROM roles WHERE organization_id = $1",
            "DELETE FROM organization_members WHERE organization_id = $1",
            "DELETE FROM organizations WHERE id = $1",
        ] {
            purge(&self.pool, statement, self.organization_id.0).await;
        }
        for person in [&self.manager, &self.plain, &self.bernard, &self.paul] {
            purge(
                &self.pool,
                "DELETE FROM users WHERE id = $1",
                person.user_id,
            )
            .await;
        }
    }
}

pub(crate) fn rolling(days: u16) -> serde_json::Value {
    json!({ "kind": "ROLLING", "every_days": days })
}

/// Today in the default organization timezone, which the use cases read.
fn today() -> NaiveDate {
    Utc::now().with_timezone(&Paris).date_naive()
}

/// A season covering today, whatever day the suite runs: the whole ISO year.
fn current_season() -> (i32, i16, i16) {
    use chrono::Datelike;
    let year = today().iso_week().year();
    let has_week_53 = NaiveDate::from_isoywd_opt(year, 53, chrono::Weekday::Mon).is_some();
    (year, 1, if has_week_53 { 53 } else { 52 })
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn spec_ent_7_the_field_proposes_and_the_manager_validates() {
    let pool = pool().await;
    let world = World::new(&pool).await;
    let site = world.seed_site("GAUME", Some("Vichy")).await;
    let season = current_season();
    let contract = world
        .seed_contract(&site, season, &[("Tonte", rolling(1))])
        .await;
    let line = contract.line_ids[0];
    let task = world.assigned_task(line, &world.bernard).await;
    let before = world.grid(season.0).await.contracts[0].lines[0].due;

    let proposal = world
        .usecase
        .propose_maintenance_intervention(ProposeInterventionCommand {
            actor: world.bernard.subject(),
            organization_id: world.organization_id,
            task_id: task.id,
            done_on: today(),
        })
        .await
        .unwrap();

    assert_eq!(proposal.status, InterventionStatus::Proposed);
    assert_eq!(proposal.task_id, Some(task.id));
    assert_eq!(proposal.proposed_by, world.bernard.member_id);
    assert_eq!(
        world.grid(season.0).await.contracts[0].lines[0].due,
        before,
        "a proposal must not move the due date"
    );

    let validated = world
        .usecase
        .validate_maintenance_intervention(
            world.organization_id,
            ValidateInterventionCommand {
                actor: world.manager.subject(),
                id: proposal.id,
            },
        )
        .await
        .unwrap();

    assert_eq!(validated.status, InterventionStatus::Validated);
    assert_eq!(validated.resolved_by, Some(world.manager.member_id));
    let after = world.grid(season.0).await.contracts[0].lines[0].due;
    assert_eq!(after.on, today().checked_add_days(Days::new(1)));
    assert_eq!(after.status, DueStatus::Planned);
    assert_eq!(
        world.event_names().await,
        [
            "maintenance_intervention.proposed",
            "maintenance_intervention.validated"
        ]
    );

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn spec_ent_8_the_field_only_proposes_on_its_own_tasks() {
    let pool = pool().await;
    let world = World::new(&pool).await;
    let site = world.seed_site("GAUME", Some("Vichy")).await;
    let contract = world
        .seed_contract(&site, current_season(), &[("Tonte", rolling(14))])
        .await;
    let pauls_task = world.assigned_task(contract.line_ids[0], &world.paul).await;

    let result = world
        .usecase
        .propose_maintenance_intervention(ProposeInterventionCommand {
            actor: world.bernard.subject(),
            organization_id: world.organization_id,
            task_id: pauls_task.id,
            done_on: today(),
        })
        .await;

    assert!(matches!(result, Err(CoreError::NotFound)), "{result:?}");
    assert_eq!(world.count_interventions().await, 0);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn a_task_that_is_not_a_maintenance_task_is_not_found_for_its_assignee() {
    let pool = pool().await;
    let world = World::new(&pool).await;
    let task = world
        .usecase
        .create_task(crate::domain::task::commands::CreateTaskCommand {
            actor: Subject::system(),
            organization_id: world.organization_id,
            parent_task_id: None,
            title: "Réunion".to_owned(),
            description: None,
            starts_at: None,
            ends_at: None,
            all_day: false,
            status: None,
            blocks_availability: false,
            customer_id: None,
            customer_context_id: None,
            quote_id: None,
            project_id: None,
            expenses_cents: 0,
            expenses_label: None,
        })
        .await
        .unwrap();
    world
        .usecase
        .patch_task(PatchTaskCommand {
            assignees: Some(vec![AssigneeRef(world.bernard.member_id)]),
            ..PatchTaskCommand::new(task.id, Subject::system())
        })
        .await
        .unwrap();

    let result = world
        .usecase
        .propose_maintenance_intervention(ProposeInterventionCommand {
            actor: world.bernard.subject(),
            organization_id: world.organization_id,
            task_id: task.id,
            done_on: today(),
        })
        .await;

    assert!(matches!(result, Err(CoreError::NotFound)), "{result:?}");

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn a_proposal_outside_the_season_or_in_the_future_is_a_conflict() {
    let pool = pool().await;
    let world = World::new(&pool).await;
    let site = world.seed_site("GAUME", Some("Vichy")).await;
    let contract = world
        .seed_contract(&site, current_season(), &[("Tonte", rolling(14))])
        .await;
    let task = world
        .assigned_task(contract.line_ids[0], &world.bernard)
        .await;
    let propose = |done_on: NaiveDate| {
        world
            .usecase
            .propose_maintenance_intervention(ProposeInterventionCommand {
                actor: world.bernard.subject(),
                organization_id: world.organization_id,
                task_id: task.id,
                done_on,
            })
    };

    let future = propose(today().checked_add_days(Days::new(3)).unwrap()).await;
    let last_year = propose(NaiveDate::from_ymd_opt(2001, 6, 1).unwrap()).await;

    assert!(matches!(future, Err(CoreError::Conflict(_))), "{future:?}");
    assert!(
        matches!(last_year, Err(CoreError::Conflict(_))),
        "{last_year:?}"
    );
    assert_eq!(world.count_interventions().await, 0);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn a_manager_entry_is_validated_at_once_and_moves_the_due_date() {
    let pool = pool().await;
    let world = World::new(&pool).await;
    let site = world.seed_site("GAUME", Some("Vichy")).await;
    let season = current_season();
    let contract = world
        .seed_contract(&site, season, &[("Tonte", rolling(1))])
        .await;

    let entry = world
        .usecase
        .record_maintenance_intervention(
            world.organization_id,
            RecordInterventionCommand {
                actor: world.manager.subject(),
                line_id: contract.line_ids[0],
                done_on: today(),
            },
        )
        .await
        .unwrap();

    assert_eq!(entry.status, InterventionStatus::Validated);
    assert_eq!(entry.task_id, None);
    assert_eq!(entry.resolved_by, Some(world.manager.member_id));
    let grid = world
        .usecase
        .maintenance_grid(world.manager.subject(), world.organization_id, season.0)
        .await
        .unwrap();
    assert_eq!(
        grid.contracts[0].lines[0].due.on,
        today().checked_add_days(Days::new(1))
    );
    assert_eq!(
        world.event_names().await,
        ["maintenance_intervention.validated"]
    );

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn rejecting_keeps_the_due_date_and_resolving_twice_is_a_conflict() {
    let pool = pool().await;
    let world = World::new(&pool).await;
    let site = world.seed_site("GAUME", Some("Vichy")).await;
    let season = current_season();
    let contract = world
        .seed_contract(&site, season, &[("Tonte", rolling(1))])
        .await;
    let line = contract.line_ids[0];
    let proposed = world
        .seed_visit(line, today(), InterventionStatus::Proposed)
        .await;
    let reject = RejectInterventionCommand {
        actor: world.manager.subject(),
        id: proposed,
        note: Some("Pas passé ce jour-là".to_owned()),
    };

    let rejected = world
        .usecase
        .reject_maintenance_intervention(world.organization_id, reject.clone())
        .await
        .unwrap();
    let again = world
        .usecase
        .reject_maintenance_intervention(world.organization_id, reject)
        .await;
    let validate_after = world
        .usecase
        .validate_maintenance_intervention(
            world.organization_id,
            ValidateInterventionCommand {
                actor: world.manager.subject(),
                id: proposed,
            },
        )
        .await;

    assert_eq!(rejected.status, InterventionStatus::Rejected);
    assert_eq!(rejected.note.as_deref(), Some("Pas passé ce jour-là"));
    assert!(matches!(again, Err(CoreError::Conflict(_))), "{again:?}");
    assert!(
        matches!(validate_after, Err(CoreError::Conflict(_))),
        "{validate_after:?}"
    );
    let grid = world
        .usecase
        .maintenance_grid(world.manager.subject(), world.organization_id, season.0)
        .await
        .unwrap();
    assert_eq!(
        grid.contracts[0].lines[0].due.on,
        Some(NaiveDate::from_isoywd_opt(season.0, 1, chrono::Weekday::Mon).unwrap())
    );
    assert_eq!(
        world.event_names().await,
        ["maintenance_intervention.rejected"]
    );

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn the_to_validate_list_holds_the_proposed_ones_oldest_first() {
    let pool = pool().await;
    let world = World::new(&pool).await;
    let site = world.seed_site("GAUME", Some("Vichy")).await;
    let contract = world
        .seed_contract(&site, current_season(), &[("Tonte", rolling(14))])
        .await;
    let line = contract.line_ids[0];
    let first = world
        .seed_visit(line, today(), InterventionStatus::Proposed)
        .await;
    let second = world
        .seed_visit(line, today(), InterventionStatus::Proposed)
        .await;
    world
        .seed_visit(line, today(), InterventionStatus::Validated)
        .await;

    let (page, total) = world
        .usecase
        .list_maintenance_interventions(
            world.manager.subject(),
            world.organization_id,
            InterventionStatus::Proposed,
            10,
            0,
        )
        .await
        .unwrap();

    assert_eq!(total, 2);
    assert_eq!(
        page.iter().map(|visit| visit.id).collect::<Vec<_>>(),
        [first, second]
    );

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn spec_ent_9_another_organization_sees_and_touches_nothing() {
    let pool = pool().await;
    let a = World::new(&pool).await;
    let b = World::new(&pool).await;
    let site = a.seed_site("GAUME", Some("Vichy")).await;
    let season = current_season();
    let contract = a
        .seed_contract(&site, season, &[("Tonte", rolling(14))])
        .await;
    let line = contract.line_ids[0];
    let visit = a
        .seed_visit(line, today(), InterventionStatus::Proposed)
        .await;
    let task = a.assigned_task(line, &a.bernard).await;

    let grid = b
        .usecase
        .maintenance_grid(b.manager.subject(), b.organization_id, season.0)
        .await
        .unwrap();
    let validate = b
        .usecase
        .validate_maintenance_intervention(
            b.organization_id,
            ValidateInterventionCommand {
                actor: b.manager.subject(),
                id: visit,
            },
        )
        .await;
    let reject = b
        .usecase
        .reject_maintenance_intervention(
            b.organization_id,
            RejectInterventionCommand {
                actor: b.manager.subject(),
                id: visit,
                note: None,
            },
        )
        .await;
    let record = b
        .usecase
        .record_maintenance_intervention(
            b.organization_id,
            RecordInterventionCommand {
                actor: b.manager.subject(),
                line_id: line,
                done_on: today(),
            },
        )
        .await;
    let plan = b.plan(&[line], 2026, 43).await;
    let propose = b
        .usecase
        .propose_maintenance_intervention(ProposeInterventionCommand {
            actor: b.bernard.subject(),
            organization_id: b.organization_id,
            task_id: task.id,
            done_on: today(),
        })
        .await;
    let listed = b
        .usecase
        .list_maintenance_interventions(
            b.manager.subject(),
            b.organization_id,
            InterventionStatus::Proposed,
            10,
            0,
        )
        .await
        .unwrap();

    assert!(grid.contracts.is_empty());
    for (what, result) in [
        ("validate", validate.map(|_| ())),
        ("reject", reject.map(|_| ())),
        ("record", record.map(|_| ())),
        ("plan", plan.map(|_| ())),
        ("propose", propose.map(|_| ())),
    ] {
        assert!(
            matches!(result, Err(CoreError::NotFound)),
            "{what}: {result:?}"
        );
    }
    assert_eq!(listed, (Vec::new(), 0));
    let untouched: String =
        sqlx::query_scalar("SELECT status FROM maintenance_interventions WHERE id = $1")
            .bind(visit.0)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(untouched, "PROPOSED");

    a.cleanup().await;
    b.cleanup().await;
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn spec_ent_10_a_member_without_planning_manage_is_forbidden() {
    let pool = pool().await;
    let world = World::new(&pool).await;
    let site = world.seed_site("GAUME", Some("Vichy")).await;
    let season = current_season();
    let contract = world
        .seed_contract(&site, season, &[("Tonte", rolling(14))])
        .await;
    let line = contract.line_ids[0];
    let visit = world
        .seed_visit(line, today(), InterventionStatus::Proposed)
        .await;
    let actor = world.plain.subject();

    let grid = world
        .usecase
        .maintenance_grid(actor.clone(), world.organization_id, season.0)
        .await
        .map(|_| ());
    let recap = world
        .usecase
        .maintenance_recap(
            actor.clone(),
            world.organization_id,
            season.0,
            IsoWeekNumber::try_from(43).unwrap(),
        )
        .await
        .map(|_| ());
    let validate = world
        .usecase
        .validate_maintenance_intervention(
            world.organization_id,
            ValidateInterventionCommand {
                actor: actor.clone(),
                id: visit,
            },
        )
        .await
        .map(|_| ());
    let reject = world
        .usecase
        .reject_maintenance_intervention(
            world.organization_id,
            RejectInterventionCommand {
                actor: actor.clone(),
                id: visit,
                note: None,
            },
        )
        .await
        .map(|_| ());
    let record = world
        .usecase
        .record_maintenance_intervention(
            world.organization_id,
            RecordInterventionCommand {
                actor: actor.clone(),
                line_id: line,
                done_on: today(),
            },
        )
        .await
        .map(|_| ());
    let list = world
        .usecase
        .list_maintenance_interventions(
            actor.clone(),
            world.organization_id,
            InterventionStatus::Proposed,
            10,
            0,
        )
        .await
        .map(|_| ());
    let plan = world
        .usecase
        .plan_maintenance_lines(PlanMaintenanceLinesCommand {
            actor,
            organization_id: world.organization_id,
            line_ids: vec![line],
            year: season.0,
            week: IsoWeekNumber::try_from(43).unwrap(),
        })
        .await
        .map(|_| ());

    for (what, result) in [
        ("grid", grid),
        ("recap", recap),
        ("validate", validate),
        ("reject", reject),
        ("record", record),
        ("list", list),
        ("plan", plan),
    ] {
        assert!(
            matches!(result, Err(CoreError::Forbidden { .. })),
            "{what}: {result:?}"
        );
    }
    let still_proposed: String =
        sqlx::query_scalar("SELECT status FROM maintenance_interventions WHERE id = $1")
            .bind(visit.0)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(still_proposed, "PROPOSED");

    world.cleanup().await;
}

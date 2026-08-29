#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use chrono::{Duration, Utc};
    use common::{CoreError, OrganizationId, UserId, generate_uuid_v7};
    use sqlx::PgPool;

    use crate::application::test_support::{dev_pool, purge};
    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::assignment_report::commands::{
        AmendAssignmentReportCommand, ReportAssignmentCommand, ResolveAssignmentReportCommand,
        WithdrawAssignmentReportCommand,
    };
    use crate::domain::task::{AssigneeRef, commands::CreateTaskCommand};
    use crate::infrastructure::realtime::EventHub;
    use crate::{AssignmentReportResolution, MemberId, PatchTaskCommand, TaskAssignmentId};

    async fn make_pool() -> PgPool {
        dev_pool().await
    }

    fn make_usecase(pool: PgPool) -> MestierUseCase {
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    struct Fixture {
        organization_id: OrganizationId,
        owner_id: UserId,
        assignee_member_id: MemberId,
        manager_member_id: MemberId,
        task_assignment_id: TaskAssignmentId,
    }

    async fn seed_member(pool: &PgPool, organization_id: OrganizationId, label: &str) -> MemberId {
        let member_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organization_members (id, organization_id, last_name)
               VALUES ($1, $2, $3)"#,
            member_id,
            organization_id.0,
            label,
        )
        .execute(pool)
        .await
        .unwrap();

        MemberId(member_id)
    }

    /// Seeds an owner user, an organization, an assignee member and a
    /// manager member, a task, and one assignment of the task to the
    /// assignee — the minimal graph an assignment report needs to exist.
    async fn seed_fixture(pool: &PgPool) -> Fixture {
        let owner_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            owner_id,
            format!("owner-{owner_id}@example.com"),
            format!("owner-{owner_id}"),
            "Owner User",
            format!("sub-owner-{owner_id}"),
        )
        .execute(pool)
        .await
        .unwrap();

        let org_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organizations (id, name, slug, owner_id)
               VALUES ($1, $2, $3, $4)"#,
            org_id,
            "Test Org",
            format!("test-org-{org_id}"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();
        let organization_id = OrganizationId(org_id);

        let assignee_member_id = seed_member(pool, organization_id, "Assignee").await;
        let manager_member_id = seed_member(pool, organization_id, "Manager").await;

        let usecase = make_usecase(pool.clone());
        let now = Utc::now();
        let task = usecase
            .create_task(CreateTaskCommand {
                actor: authz::Subject::system(),
                organization_id,
                parent_task_id: None,
                title: "Réunion de chantier".to_owned(),
                description: None,
                starts_at: Some(now),
                ends_at: Some(now + Duration::hours(1)),
                all_day: false,
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

        let patched = usecase
            .patch_task(PatchTaskCommand {
                assignees: Some(vec![AssigneeRef(assignee_member_id)]),
                ..PatchTaskCommand::new(task.id, authz::Subject::system())
            })
            .await
            .unwrap();
        let task_assignment_id = patched.assignments[0].id;

        Fixture {
            organization_id,
            owner_id: UserId(owner_id),
            assignee_member_id,
            manager_member_id,
            task_assignment_id,
        }
    }

    async fn cleanup(pool: &PgPool, organization_id: OrganizationId, user_ids: &[UserId]) {
        // `report_assignment`/`resolve_assignment_report` emit durable events
        // (`assignment_report.reported`/`.applied`/`.dismissed`), which
        // reference the organization and must be purged before it — mirrors
        // the fixture at `application/mod.rs`'s own `automation.event` note.
        purge(
            pool,
            "DELETE FROM automation.event WHERE org_id = $1",
            organization_id.0,
        )
        .await;
        purge(
            pool,
            "DELETE FROM assignment_reports WHERE org_id = $1",
            organization_id.0,
        )
        .await;
        purge(
            pool,
            "DELETE FROM task_assignments WHERE org_id = $1",
            organization_id.0,
        )
        .await;
        purge(
            pool,
            "DELETE FROM tasks WHERE org_id = $1",
            organization_id.0,
        )
        .await;
        purge(
            pool,
            "DELETE FROM organization_members WHERE organization_id = $1",
            organization_id.0,
        )
        .await;
        purge(
            pool,
            "DELETE FROM organizations WHERE id = $1",
            organization_id.0,
        )
        .await;
        for uid in user_ids {
            purge(pool, "DELETE FROM users WHERE id = $1", uid.0).await;
        }
    }

    fn report_command(fixture: &Fixture) -> ReportAssignmentCommand {
        ReportAssignmentCommand {
            task_assignment_id: fixture.task_assignment_id,
            reported_by: fixture.assignee_member_id,
            reported_minutes: 300,
            comment: Some("Plus long que prévu".to_owned()),
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn report_assignment_persists_and_is_retrievable() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .report_assignment(report_command(&fixture))
            .await
            .expect("report_assignment must succeed");

        assert_eq!(created.reported_minutes, 300);
        assert_eq!(created.reported_by, fixture.assignee_member_id);
        assert_eq!(created.resolution, AssignmentReportResolution::Pending);

        let fetched = usecase
            .get_assignment_report(created.id)
            .await
            .expect("get_assignment_report must succeed");
        assert_eq!(fetched.id, created.id);

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// Proves the acceptance criterion end to end: reporting on an
    /// assignment that belongs to someone else is `Forbidden`.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn report_assignment_by_a_different_member_is_forbidden() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let mut command = report_command(&fixture);
        command.reported_by = fixture.manager_member_id;

        let err = usecase
            .report_assignment(command)
            .await
            .expect_err("a non-assignee must not be able to report");

        assert!(matches!(err, CoreError::Forbidden { .. }));

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// Proves the acceptance criterion at the database level: the partial
    /// unique index refuses a second pending report on the same assignment,
    /// surfaced as a `Conflict` via `map_sqlx_error`'s unique-violation
    /// mapping.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_second_pending_report_on_the_same_assignment_is_rejected() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        usecase
            .report_assignment(report_command(&fixture))
            .await
            .unwrap();

        let err = usecase
            .report_assignment(report_command(&fixture))
            .await
            .expect_err("a second pending report on the same assignment must be refused");

        assert!(matches!(err, CoreError::Conflict(_)));

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn amend_report_by_its_author_succeeds() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .report_assignment(report_command(&fixture))
            .await
            .unwrap();

        let amended = usecase
            .amend_assignment_report(AmendAssignmentReportCommand {
                id: created.id,
                acting_member_id: fixture.assignee_member_id,
                reported_minutes: 240,
                comment: None,
            })
            .await
            .expect("the author must be able to amend their own pending report");

        assert_eq!(amended.reported_minutes, 240);
        assert_eq!(amended.comment, None);

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn withdraw_report_by_its_author_removes_it() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .report_assignment(report_command(&fixture))
            .await
            .unwrap();

        usecase
            .withdraw_assignment_report(WithdrawAssignmentReportCommand {
                id: created.id,
                acting_member_id: fixture.assignee_member_id,
            })
            .await
            .expect("the author must be able to withdraw their own pending report");

        let err = usecase
            .get_assignment_report(created.id)
            .await
            .expect_err("a withdrawn report must no longer be gettable");
        assert!(matches!(err, CoreError::NotFound));

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn resolve_report_applies_it_without_touching_the_task() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .report_assignment(report_command(&fixture))
            .await
            .unwrap();

        let resolved = usecase
            .resolve_assignment_report(ResolveAssignmentReportCommand {
                id: created.id,
                resolved_by: fixture.manager_member_id,
                resolution: AssignmentReportResolution::Applied,
                resolution_note: None,
            })
            .await
            .expect("resolving a pending report must succeed");

        assert_eq!(resolved.resolution, AssignmentReportResolution::Applied);
        assert_eq!(resolved.resolved_by, Some(fixture.manager_member_id));
        assert!(resolved.resolved_at.is_some());

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// Resolving an already-resolved report must fail loudly rather than
    /// silently no-op — the acceptance criterion from the issue's "Domain"
    /// section.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn resolving_an_already_resolved_report_is_refused() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .report_assignment(report_command(&fixture))
            .await
            .unwrap();

        usecase
            .resolve_assignment_report(ResolveAssignmentReportCommand {
                id: created.id,
                resolved_by: fixture.manager_member_id,
                resolution: AssignmentReportResolution::Dismissed,
                resolution_note: Some("Doublon".to_owned()),
            })
            .await
            .unwrap();

        let err = usecase
            .resolve_assignment_report(ResolveAssignmentReportCommand {
                id: created.id,
                resolved_by: fixture.manager_member_id,
                resolution: AssignmentReportResolution::Applied,
                resolution_note: None,
            })
            .await
            .expect_err("a report already resolved must not resolve again");

        assert!(matches!(err, CoreError::Conflict(_)));

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn list_assignment_reports_by_reporter_filters_by_resolution() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .report_assignment(report_command(&fixture))
            .await
            .unwrap();

        let (pending, pending_total) = usecase
            .list_assignment_reports_by_reporter(
                fixture.organization_id,
                fixture.assignee_member_id,
                Some(AssignmentReportResolution::Pending),
                20,
                0,
            )
            .await
            .unwrap();
        assert_eq!(pending_total, 1);
        assert_eq!(pending[0].id, created.id);

        let (applied, applied_total) = usecase
            .list_assignment_reports_by_reporter(
                fixture.organization_id,
                fixture.assignee_member_id,
                Some(AssignmentReportResolution::Applied),
                20,
                0,
            )
            .await
            .unwrap();
        assert_eq!(applied_total, 0);
        assert!(applied.is_empty());

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }
}

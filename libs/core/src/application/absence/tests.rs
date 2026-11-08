#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {

    use chrono::{Duration, Utc};
    use common::{OrganizationId, UserId, generate_uuid_v7};
    use sqlx::PgPool;

    use crate::application::test_support::{dev_pool, purge};
    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::absence::commands::{
        AbsenceSchedule, CreateAbsenceCommand, DecideAbsenceCommand, PatchAbsenceCommand,
        RejectAbsenceCommand,
    };
    use crate::domain::absence_type::commands::{
        AbsenceTypeDraft, ArchiveAbsenceTypeCommand, CreateAbsenceTypeCommand,
        ListAbsenceTypesCommand,
    };
    use crate::infrastructure::realtime::EventHub;
    use crate::{AbsenceError, AbsenceStatus, AbsenceTypeId, CountMode, DayPart};
    use crate::{AbsenceId, AbsenceKind, EmployeeId, MemberId};

    async fn make_pool() -> PgPool {
        dev_pool().await
    }

    fn make_usecase(pool: PgPool) -> MestierUseCase {
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    struct Fixture {
        organization_id: OrganizationId,
        member_id: MemberId,
        #[allow(dead_code)]
        employee_id: EmployeeId,
        owner_id: UserId,
    }

    /// Seeds an owner user, an organization and an employee — the minimal
    /// graph an absence needs to exist.
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

        let member_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organization_members (id, organization_id, last_name)
               VALUES ($1, $2, $3)"#,
            member_id,
            org_id,
            "Alice Employee",
        )
        .execute(pool)
        .await
        .unwrap();

        let employee_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO employees (id, org_id, member_id, hourly_rate_cents, weekly_contract_minutes)
               VALUES ($1, $2, $3, $4, $5)"#,
            employee_id,
            org_id,
            member_id,
            3500,
            2100,
        )
        .execute(pool)
        .await
        .unwrap();

        Fixture {
            organization_id: OrganizationId(org_id),
            member_id: MemberId(member_id),
            employee_id: EmployeeId(employee_id),
            owner_id: UserId(owner_id),
        }
    }

    /// Removes everything seeded under `organization_id`, cascading to
    /// employees/absences, plus the loose user rows that outlive
    /// the organization.
    async fn cleanup(pool: &PgPool, organization_id: OrganizationId, user_ids: &[UserId]) {
        purge(
            pool,
            "DELETE FROM absences WHERE org_id = $1",
            organization_id.0,
        )
        .await;
        purge(
            pool,
            "DELETE FROM absence_types WHERE org_id = $1",
            organization_id.0,
        )
        .await;
        purge(
            pool,
            "DELETE FROM employees WHERE org_id = $1",
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

    fn create_command(fixture: &Fixture) -> CreateAbsenceCommand {
        let now = Utc::now();
        CreateAbsenceCommand {
            actor: authz::Subject::system(),
            organization_id: fixture.organization_id,
            member_id: fixture.member_id,
            kind: Some(AbsenceKind::Leave),
            type_id: None,
            schedule: AbsenceSchedule::Window {
                starts_at: now,
                ends_at: now + Duration::hours(2),
                all_day: false,
            },
            note: Some("Congés payés".to_owned()),
            proof_file_key: None,
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn create_absence_persists_and_is_retrievable() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .create_absence(create_command(&fixture))
            .await
            .expect("create_absence must succeed")
            .expect("the request is valid");

        assert_eq!(created.kind, AbsenceKind::Leave);
        assert_eq!(created.member_id, fixture.member_id);

        let fetched = usecase
            .get_absence(created.id)
            .await
            .expect("get_absence must succeed");
        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.note.as_deref(), Some("Congés payés"));

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn list_absences_returns_only_the_organizations_own() {
        let pool = make_pool().await;
        let fixture_a = seed_fixture(&pool).await;
        let fixture_b = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        usecase
            .create_absence(create_command(&fixture_a))
            .await
            .unwrap()
            .unwrap();
        usecase
            .create_absence(create_command(&fixture_b))
            .await
            .unwrap()
            .unwrap();

        let (items, total) = usecase
            .list_absences(fixture_a.organization_id, 20, 0)
            .await
            .expect("list_absences must succeed");

        assert_eq!(total, 1);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].organization_id, fixture_a.organization_id);

        cleanup(&pool, fixture_a.organization_id, &[fixture_a.owner_id]).await;
        cleanup(&pool, fixture_b.organization_id, &[fixture_b.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn patch_absence_reschedules_and_changes_kind() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .create_absence(create_command(&fixture))
            .await
            .unwrap()
            .unwrap();

        let new_starts_at = created.starts_at + Duration::days(1);
        let new_ends_at = created.ends_at + Duration::days(1);

        let mut patch = PatchAbsenceCommand::new(created.id, authz::Subject::system());
        patch.starts_at = Some(new_starts_at);
        patch.ends_at = Some(new_ends_at);
        patch.kind = Some(AbsenceKind::Sick);
        patch.note = Some(None);

        let updated = usecase
            .patch_absence(patch)
            .await
            .expect("patch_absence must succeed");

        assert_eq!(updated.starts_at, new_starts_at);
        assert_eq!(updated.ends_at, new_ends_at);
        assert_eq!(updated.kind, AbsenceKind::Sick);
        assert!(updated.note.is_none());

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// Proves the promise a `deleted_at` column makes: soft-deleting an
    /// absence removes it from reads (`get_absence`, `list_absences`) while
    /// the row itself survives in the table, deleted_at populated.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn soft_delete_absence_hides_it_from_reads_but_keeps_the_row_in_the_database() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .create_absence(create_command(&fixture))
            .await
            .unwrap()
            .unwrap();

        usecase
            .soft_delete_absence(authz::Subject::system(), created.id)
            .await
            .expect("soft_delete_absence must succeed");

        let err = usecase
            .get_absence(created.id)
            .await
            .expect_err("a soft-deleted absence must not be gettable");
        assert!(matches!(err, common::CoreError::NotFound));

        let (items, total) = usecase
            .list_absences(fixture.organization_id, 20, 0)
            .await
            .unwrap();
        assert_eq!(total, 0);
        assert!(items.is_empty());

        let row = sqlx::query!(
            r#"SELECT deleted_at FROM absences WHERE id = $1"#,
            created.id.0,
        )
        .fetch_one(&pool)
        .await
        .expect("the row must still physically exist after a soft delete");
        assert!(
            row.deleted_at.is_some(),
            "deleted_at must be populated by the soft delete"
        );

        let missing = usecase
            .soft_delete_absence(authz::Subject::system(), AbsenceId(generate_uuid_v7()))
            .await
            .expect_err("deleting an unknown absence must fail");
        assert!(matches!(missing, common::CoreError::NotFound));

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    fn draft(name: &str, needs_approval: bool, needs_proof: bool) -> AbsenceTypeDraft {
        AbsenceTypeDraft {
            name: name.to_owned(),
            hue: 120,
            count_mode: CountMode::Working,
            account_id: None,
            paid: true,
            needs_approval,
            needs_proof,
            kind: AbsenceKind::Leave,
        }
    }

    async fn seed_type(
        usecase: &MestierUseCase,
        fixture: &Fixture,
        name: &str,
        needs_approval: bool,
        needs_proof: bool,
    ) -> AbsenceTypeId {
        usecase
            .create_absence_type(CreateAbsenceTypeCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                draft: draft(name, needs_approval, needs_proof),
            })
            .await
            .unwrap()
            .unwrap()
            .id
    }

    fn days_command(fixture: &Fixture, type_id: AbsenceTypeId) -> CreateAbsenceCommand {
        CreateAbsenceCommand {
            kind: None,
            type_id: Some(type_id),
            schedule: AbsenceSchedule::Days {
                start_date: chrono::NaiveDate::from_ymd_opt(2026, 7, 6).unwrap(),
                start_part: DayPart::Afternoon,
                end_date: chrono::NaiveDate::from_ymd_opt(2026, 7, 8).unwrap(),
                end_part: DayPart::Morning,
            },
            ..create_command(fixture)
        }
    }

    fn decide(fixture: &Fixture, id: AbsenceId) -> DecideAbsenceCommand {
        DecideAbsenceCommand {
            actor: authz::Subject::system(),
            organization_id: fixture.organization_id,
            id,
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn spec_rh_3_2_and_3_8_a_day_request_is_pending_with_the_organization_timezone_window() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());
        let needing = seed_type(&usecase, &fixture, "Besoin d'accord", true, false).await;
        let free = seed_type(&usecase, &fixture, "Sans accord", false, false).await;

        let pending = usecase
            .create_absence(days_command(&fixture, needing))
            .await
            .unwrap()
            .unwrap();
        let approved = usecase
            .create_absence(days_command(&fixture, free))
            .await
            .unwrap()
            .unwrap();

        assert_eq!(pending.status, AbsenceStatus::Pending);
        assert_eq!(approved.status, AbsenceStatus::Approved);
        assert!(approved.decided_by_member_id.is_none());
        assert_eq!(
            pending.starts_at,
            chrono::DateTime::parse_from_rfc3339("2026-07-06T10:00:00Z").unwrap()
        );
        assert_eq!(
            pending.ends_at,
            chrono::DateTime::parse_from_rfc3339("2026-07-08T10:00:00Z").unwrap()
        );
        assert!(!pending.all_day);

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn spec_rh_3_3_approving_then_deciding_again_is_refused() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());
        let needing = seed_type(&usecase, &fixture, "Besoin d'accord", true, false).await;
        let created = usecase
            .create_absence(days_command(&fixture, needing))
            .await
            .unwrap()
            .unwrap();

        let approved = usecase
            .approve_absence(decide(&fixture, created.id))
            .await
            .unwrap()
            .unwrap();
        let again = usecase
            .approve_absence(decide(&fixture, created.id))
            .await
            .unwrap();
        let rejected = usecase
            .reject_absence(RejectAbsenceCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                id: created.id,
                comment: "non".to_owned(),
            })
            .await
            .unwrap();

        assert_eq!(approved.status, AbsenceStatus::Approved);
        assert!(approved.decided_at.is_some());
        assert_eq!(again, Err(AbsenceError::AlreadyDecided));
        assert_eq!(rejected, Err(AbsenceError::AlreadyDecided));

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn spec_rh_3_4_and_3_5_rejection_needs_a_comment_and_proof_gates_approval() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());
        let proof_type = seed_type(&usecase, &fixture, "Sur justificatif", true, true).await;
        let created = usecase
            .create_absence(days_command(&fixture, proof_type))
            .await
            .unwrap()
            .unwrap();

        let blank = usecase
            .reject_absence(RejectAbsenceCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                id: created.id,
                comment: "  ".to_owned(),
            })
            .await
            .unwrap();
        let without_proof = usecase
            .approve_absence(decide(&fixture, created.id))
            .await
            .unwrap();

        assert_eq!(blank, Err(AbsenceError::CommentRequired));
        assert_eq!(without_proof, Err(AbsenceError::ProofRequired));

        let mut patch = PatchAbsenceCommand::new(created.id, authz::Subject::system());
        patch.proof_file_key = Some(Some("absences/justificatif.pdf".to_owned()));
        usecase.patch_absence(patch).await.unwrap();
        let approved = usecase
            .approve_absence(decide(&fixture, created.id))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(approved.status, AbsenceStatus::Approved);

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn spec_rh_3_6_an_archived_type_is_refused_for_new_absences_and_stays_on_past_ones() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());
        let type_id = seed_type(&usecase, &fixture, "Ancien motif", true, false).await;
        let past = usecase
            .create_absence(days_command(&fixture, type_id))
            .await
            .unwrap()
            .unwrap();

        usecase
            .archive_absence_type(ArchiveAbsenceTypeCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                id: type_id,
                archived: true,
            })
            .await
            .unwrap();

        let refused = usecase
            .create_absence(days_command(&fixture, type_id))
            .await
            .unwrap();
        assert_eq!(refused, Err(AbsenceError::TypeArchived));
        let kept = usecase.get_absence(past.id).await.unwrap();
        assert_eq!(kept.type_id, type_id);
        let types = usecase
            .list_absence_types(ListAbsenceTypesCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
            })
            .await
            .unwrap();
        assert!(
            types
                .iter()
                .any(|t| t.id == type_id && t.archived_at.is_some())
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn cancelling_by_a_system_caller_needs_no_creator_and_the_other_organization_sees_not_found()
     {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let other = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());
        let created = usecase
            .create_absence(create_command(&fixture))
            .await
            .unwrap()
            .unwrap();

        let foreign = usecase
            .cancel_absence(decide(&other, created.id))
            .await
            .unwrap_err();
        let cancelled = usecase
            .cancel_absence(decide(&fixture, created.id))
            .await
            .unwrap()
            .unwrap();

        assert!(matches!(foreign, common::CoreError::NotFound));
        assert_eq!(cancelled.status, AbsenceStatus::Cancelled);

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
        cleanup(&pool, other.organization_id, &[other.owner_id]).await;
    }
}

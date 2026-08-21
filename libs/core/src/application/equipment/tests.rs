#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {

    use chrono::{Duration, Utc};
    use common::{OrganizationId, UserId, generate_uuid_v7};
    use sqlx::PgPool;

    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::equipment::commands::CreateEquipmentCommand;
    use crate::infrastructure::realtime::EventHub;
    use crate::{CreateTaskCommand, EquipmentId, PatchTaskCommand};

    async fn make_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run equipment integration tests");
        PgPool::connect(&url).await.unwrap()
    }

    fn make_usecase(pool: PgPool) -> MestierUseCase {
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    struct Fixture {
        organization_id: OrganizationId,
        owner_id: UserId,
    }

    /// Seeds an owner user and an organization — the minimal graph a task
    /// and its equipment need to exist. Mirrors `task_label`'s own fixture.
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

        Fixture {
            organization_id: OrganizationId(org_id),
            owner_id: UserId(owner_id),
        }
    }

    /// Removes everything seeded under `organization_id`: equipment links
    /// cascade from `tasks`/`equipment` themselves, so only those two plus
    /// the organization and its loose users need explicit cleanup.
    async fn cleanup(pool: &PgPool, organization_id: OrganizationId, user_ids: &[UserId]) {
        sqlx::query!(
            "DELETE FROM task_assignments WHERE org_id = $1",
            organization_id.0
        )
        .execute(pool)
        .await
        .ok();
        sqlx::query!("DELETE FROM tasks WHERE org_id = $1", organization_id.0)
            .execute(pool)
            .await
            .ok();
        sqlx::query!("DELETE FROM equipment WHERE org_id = $1", organization_id.0)
            .execute(pool)
            .await
            .ok();
        sqlx::query!("DELETE FROM organizations WHERE id = $1", organization_id.0)
            .execute(pool)
            .await
            .ok();
        for uid in user_ids {
            sqlx::query!("DELETE FROM users WHERE id = $1", uid.0)
                .execute(pool)
                .await
                .ok();
        }
    }

    fn create_equipment_command(
        organization_id: OrganizationId,
        name: &str,
    ) -> CreateEquipmentCommand {
        CreateEquipmentCommand {
            organization_id,
            name: name.to_owned(),
            hourly_rate_cents: 1200,
        }
    }

    fn create_task_command(fixture: &Fixture, title: &str) -> CreateTaskCommand {
        let now = Utc::now();
        CreateTaskCommand {
            organization_id: fixture.organization_id,
            parent_task_id: None,
            title: title.to_owned(),
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
        }
    }

    async fn count_links_for_equipment(pool: &PgPool, equipment_id: EquipmentId) -> i64 {
        sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM task_equipment_links WHERE equipment_id = $1"#,
            equipment_id.0,
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    /// Proves `PATCH /tasks`'s `equipment_ids` contract: the complete
    /// replacement list, never a delta — same semantics as `label_ids`.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn patch_task_equipment_ids_replaces_the_complete_set() {
        let pool = make_pool().await;
        let fixture = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let task = usecase
            .create_task(create_task_command(&fixture, "Chantier"))
            .await
            .unwrap();
        let truck = usecase
            .create_equipment(create_equipment_command(fixture.organization_id, "Camion"))
            .await
            .unwrap();
        let mower = usecase
            .create_equipment(create_equipment_command(
                fixture.organization_id,
                "Tondeuse",
            ))
            .await
            .unwrap();
        let trailer = usecase
            .create_equipment(create_equipment_command(
                fixture.organization_id,
                "Remorque",
            ))
            .await
            .unwrap();

        let mut first_patch = PatchTaskCommand::new(task.id);
        first_patch.equipment_ids = Some(vec![truck.id, mower.id]);
        usecase.patch_task(first_patch).await.unwrap();

        assert_eq!(count_links_for_equipment(&pool, truck.id).await, 1);
        assert_eq!(count_links_for_equipment(&pool, mower.id).await, 1);
        assert_eq!(count_links_for_equipment(&pool, trailer.id).await, 0);

        // A second PATCH names only `trailer` — `truck`/`mower` must be
        // dropped, not merged with the new set.
        let mut second_patch = PatchTaskCommand::new(task.id);
        second_patch.equipment_ids = Some(vec![trailer.id]);
        usecase.patch_task(second_patch).await.unwrap();

        assert_eq!(
            count_links_for_equipment(&pool, truck.id).await,
            0,
            "equipment_ids replaces the set — truck must be gone"
        );
        assert_eq!(count_links_for_equipment(&pool, mower.id).await, 0);
        assert_eq!(count_links_for_equipment(&pool, trailer.id).await, 1);

        // A patch that never mentions `equipment_ids` leaves the current set
        // untouched — mirrors `assignees: None`.
        let mut untouched_patch = PatchTaskCommand::new(task.id);
        untouched_patch.title = Some("Chantier renommé".to_owned());
        usecase.patch_task(untouched_patch).await.unwrap();
        assert_eq!(count_links_for_equipment(&pool, trailer.id).await, 1);

        // An empty `equipment_ids` clears the set entirely.
        let mut clearing_patch = PatchTaskCommand::new(task.id);
        clearing_patch.equipment_ids = Some(Vec::new());
        usecase.patch_task(clearing_patch).await.unwrap();
        assert_eq!(
            count_links_for_equipment(&pool, trailer.id).await,
            0,
            "an empty equipment_ids must remove every equipment link from the task"
        );

        cleanup(&pool, fixture.organization_id, &[fixture.owner_id]).await;
    }

    /// Proves the cross-organization guard: an id naming real equipment from
    /// a different organization is rejected as `NotFound` before anything is
    /// written, exactly like a foreign `label_id`.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn patch_task_equipment_ids_rejects_equipment_from_another_organization() {
        let pool = make_pool().await;
        let fixture_a = seed_fixture(&pool).await;
        let fixture_b = seed_fixture(&pool).await;
        let usecase = make_usecase(pool.clone());

        let task = usecase
            .create_task(create_task_command(&fixture_a, "Chantier"))
            .await
            .unwrap();
        let foreign_equipment = usecase
            .create_equipment(create_equipment_command(
                fixture_b.organization_id,
                "Camion",
            ))
            .await
            .unwrap();

        let mut patch = PatchTaskCommand::new(task.id);
        patch.equipment_ids = Some(vec![foreign_equipment.id]);
        let err = usecase.patch_task(patch).await.unwrap_err();

        assert!(matches!(err, common::CoreError::NotFound));
        assert_eq!(
            count_links_for_equipment(&pool, foreign_equipment.id).await,
            0
        );

        cleanup(&pool, fixture_a.organization_id, &[fixture_a.owner_id]).await;
        cleanup(&pool, fixture_b.organization_id, &[fixture_b.owner_id]).await;
    }
}

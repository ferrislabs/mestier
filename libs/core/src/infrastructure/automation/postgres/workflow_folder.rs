use chrono::{DateTime, Utc};
use common::{CoreError, OrganizationId};
use mestier_macros::repository;
use uuid::Uuid;

use crate::{
    domain::automation::{
        folder::{FolderName, FolderRefusal, WorkflowFolder},
        ports::WorkflowFolderRepository,
    },
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

const NAME_INDEX: &str = "uq_automation_workflow_folder_name";

#[repository(domain = WorkflowFolder, backend = Postgres)]
pub struct PgWorkflowFolderRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgWorkflowFolderRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

struct WorkflowFolderRow {
    id: Uuid,
    org_id: Uuid,
    parent_id: Option<Uuid>,
    name: String,
    created_at: DateTime<Utc>,
}

impl TryFrom<WorkflowFolderRow> for WorkflowFolder {
    type Error = CoreError;

    fn try_from(row: WorkflowFolderRow) -> Result<Self, Self::Error> {
        let name = FolderName::try_from(row.name).map_err(|e| {
            CoreError::Internal(format!("invalid workflow folder in database: {e}"))
        })?;

        Ok(Self {
            id: row.id,
            org_id: OrganizationId(row.org_id),
            parent_id: row.parent_id,
            name,
            created_at: row.created_at,
        })
    }
}

fn map_write_error(error: sqlx::Error) -> CoreError {
    match &error {
        sqlx::Error::Database(database)
            if database.is_unique_violation() && database.constraint() == Some(NAME_INDEX) =>
        {
            FolderRefusal::NameTaken.into()
        }
        sqlx::Error::Database(database) if database.is_foreign_key_violation() => {
            FolderRefusal::Missing.into()
        }
        _ => map_sqlx_error(error),
    }
}

fn map_delete_error(error: sqlx::Error) -> CoreError {
    match &error {
        sqlx::Error::Database(database) if database.is_foreign_key_violation() => {
            FolderRefusal::NotEmpty.into()
        }
        _ => map_sqlx_error(error),
    }
}

impl<'tx> WorkflowFolderRepository for PgWorkflowFolderRepository<'tx> {
    async fn insert(&mut self, folder: &WorkflowFolder) -> Result<WorkflowFolder, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            WorkflowFolderRow,
            r#"INSERT INTO automation.workflow_folder (id, org_id, parent_id, name, created_at)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING id, org_id, parent_id, name, created_at"#,
            folder.id,
            folder.org_id.0,
            folder.parent_id,
            folder.name.as_str(),
            folder.created_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_write_error)?;

        row.try_into()
    }

    async fn find_by_id(
        &mut self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Option<WorkflowFolder>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            WorkflowFolderRow,
            r#"SELECT id, org_id, parent_id, name, created_at
               FROM automation.workflow_folder
               WHERE org_id = $1 AND id = $2"#,
            org_id.0,
            id,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(WorkflowFolder::try_from).transpose()
    }

    async fn list_by_organization(
        &mut self,
        org_id: OrganizationId,
    ) -> Result<Vec<WorkflowFolder>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            WorkflowFolderRow,
            r#"SELECT id, org_id, parent_id, name, created_at
               FROM automation.workflow_folder
               WHERE org_id = $1
               ORDER BY lower(name), id"#,
            org_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(WorkflowFolder::try_from).collect()
    }

    async fn lock_by_organization(
        &mut self,
        org_id: OrganizationId,
    ) -> Result<Vec<WorkflowFolder>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            WorkflowFolderRow,
            r#"SELECT id, org_id, parent_id, name, created_at
               FROM automation.workflow_folder
               WHERE org_id = $1
               ORDER BY lower(name), id
               FOR UPDATE"#,
            org_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(WorkflowFolder::try_from).collect()
    }

    async fn update(
        &mut self,
        org_id: OrganizationId,
        folder: &WorkflowFolder,
    ) -> Result<WorkflowFolder, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            WorkflowFolderRow,
            r#"UPDATE automation.workflow_folder
               SET name = $3, parent_id = $4
               WHERE org_id = $1 AND id = $2
               RETURNING id, org_id, parent_id, name, created_at"#,
            org_id.0,
            folder.id,
            folder.name.as_str(),
            folder.parent_id,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_write_error)?;

        row.map(WorkflowFolder::try_from)
            .transpose()?
            .ok_or(CoreError::NotFound)
    }

    async fn delete(&mut self, org_id: OrganizationId, id: Uuid) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let outcome = sqlx::query!(
            "DELETE FROM automation.workflow_folder WHERE org_id = $1 AND id = $2",
            org_id.0,
            id,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_delete_error)?;

        if outcome.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use common::generate_uuid_v7;
    use sqlx::PgPool;

    use super::*;
    use crate::application::test_support::{automation_pool, now_storable};
    use crate::domain::automation::{ports::WorkflowRepository, workflow::Workflow};
    use crate::infrastructure::automation::postgres::PgWorkflowRepository;
    use crate::infrastructure::postgres::with_tx;

    async fn make_pool() -> PgPool {
        automation_pool().await
    }

    async fn seed_organization(pool: &PgPool, label: &str) -> OrganizationId {
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
            format!("{label} Org"),
            format!("{label}-{org_id}"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();

        OrganizationId(org_id)
    }

    fn folder(org_id: OrganizationId, parent_id: Option<Uuid>, name: &str) -> WorkflowFolder {
        WorkflowFolder {
            id: generate_uuid_v7(),
            org_id,
            parent_id,
            name: FolderName::try_from(name.to_owned()).unwrap(),
            created_at: now_storable(),
        }
    }

    async fn insert(pool: &PgPool, folder: &WorkflowFolder) -> Result<WorkflowFolder, CoreError> {
        with_tx(pool, async |tx| {
            let mut repo = PgWorkflowFolderRepository::new(&tx);
            repo.insert(folder).await
        })
        .await
    }

    async fn seed_workflow_in(
        pool: &PgPool,
        org_id: OrganizationId,
        folder_id: Uuid,
        archived: bool,
    ) -> Uuid {
        let now = now_storable();
        let workflow = Workflow {
            id: generate_uuid_v7(),
            org_id,
            name: "Filed".to_owned(),
            description: None,
            enabled: !archived,
            current_version_id: None,
            layout: None,
            folder_id: Some(folder_id),
            tags: Default::default(),
            archived_at: archived.then_some(now),
            created_at: now,
            updated_at: now,
        };
        with_tx(pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.insert(&workflow).await
        })
        .await
        .unwrap()
        .id
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_inserted_folder_is_found_in_its_own_organization_only() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "folder-find").await;
        let stranger = seed_organization(&pool, "folder-find-stranger").await;
        let written = folder(org_id, None, "Facturation");

        let inserted = insert(&pool, &written).await.unwrap();
        assert_eq!(inserted, written);

        let (mine, theirs) = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowFolderRepository::new(&tx);
            let mine = repo.find_by_id(org_id, written.id).await?;
            let theirs = repo.find_by_id(stranger, written.id).await?;
            Ok::<_, CoreError>((mine, theirs))
        })
        .await
        .unwrap();

        assert_eq!(mine, Some(written));
        assert_eq!(theirs, None);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn listing_returns_only_the_organizations_folders_ordered_by_name() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "folder-list").await;
        let other = seed_organization(&pool, "folder-list-other").await;
        let root = insert(&pool, &folder(org_id, None, "zeta")).await.unwrap();
        insert(&pool, &folder(org_id, Some(root.id), "Alpha"))
            .await
            .unwrap();
        insert(&pool, &folder(org_id, None, "beta")).await.unwrap();
        insert(&pool, &folder(other, None, "Not mine"))
            .await
            .unwrap();

        let listed = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowFolderRepository::new(&tx);
            repo.list_by_organization(org_id).await
        })
        .await
        .unwrap();

        let names: Vec<&str> = listed.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, ["Alpha", "beta", "zeta"]);
        assert!(listed.iter().all(|f| f.org_id == org_id));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn locking_returns_the_same_rows_as_listing() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "folder-lock").await;
        insert(&pool, &folder(org_id, None, "A")).await.unwrap();
        insert(&pool, &folder(org_id, None, "B")).await.unwrap();

        let (listed, locked) = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowFolderRepository::new(&tx);
            let listed = repo.list_by_organization(org_id).await?;
            let locked = repo.lock_by_organization(org_id).await?;
            Ok::<_, CoreError>((listed, locked))
        })
        .await
        .unwrap();

        assert_eq!(listed, locked);
        assert_eq!(locked.len(), 2);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_name_is_unique_per_parent_ignoring_case_including_at_the_root() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "folder-unique").await;
        let root = insert(&pool, &folder(org_id, None, "Facturation"))
            .await
            .unwrap();
        insert(&pool, &folder(org_id, Some(root.id), "Relances"))
            .await
            .unwrap();

        let at_root = insert(&pool, &folder(org_id, None, "FACTURATION")).await;
        let below = insert(&pool, &folder(org_id, Some(root.id), "relances")).await;

        assert!(
            matches!(&at_root, Err(CoreError::Conflict(m)) if FolderRefusal::from_conflict(m) == Some(FolderRefusal::NameTaken))
        );
        assert!(
            matches!(&below, Err(CoreError::Conflict(m)) if FolderRefusal::from_conflict(m) == Some(FolderRefusal::NameTaken))
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_same_name_is_allowed_under_another_parent_and_in_another_organization() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "folder-same-name").await;
        let other = seed_organization(&pool, "folder-same-name-other").await;
        let first = insert(&pool, &folder(org_id, None, "A")).await.unwrap();
        let second = insert(&pool, &folder(org_id, None, "B")).await.unwrap();

        insert(&pool, &folder(org_id, Some(first.id), "Shared"))
            .await
            .unwrap();
        insert(&pool, &folder(org_id, Some(second.id), "Shared"))
            .await
            .unwrap();
        insert(&pool, &folder(org_id, None, "Shared"))
            .await
            .unwrap();
        insert(&pool, &folder(other, None, "Shared")).await.unwrap();
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn updating_renames_and_moves_a_folder() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "folder-update").await;
        let target = insert(&pool, &folder(org_id, None, "Target"))
            .await
            .unwrap();
        let moved = insert(&pool, &folder(org_id, None, "Old")).await.unwrap();

        let updated = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowFolderRepository::new(&tx);
            repo.update(
                org_id,
                &WorkflowFolder {
                    name: FolderName::try_from("New".to_owned()).unwrap(),
                    parent_id: Some(target.id),
                    ..moved.clone()
                },
            )
            .await
        })
        .await
        .unwrap();

        assert_eq!(updated.name.as_str(), "New");
        assert_eq!(updated.parent_id, Some(target.id));
        assert_eq!(updated.created_at, moved.created_at);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn updating_a_folder_of_another_organization_is_not_found() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "folder-update-owner").await;
        let stranger = seed_organization(&pool, "folder-update-stranger").await;
        let written = insert(&pool, &folder(org_id, None, "Mine")).await.unwrap();

        let outcome = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowFolderRepository::new(&tx);
            repo.update(stranger, &written).await
        })
        .await;

        assert!(matches!(outcome, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn deleting_an_empty_folder_removes_it() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "folder-delete").await;
        let written = insert(&pool, &folder(org_id, None, "Empty")).await.unwrap();

        let found = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowFolderRepository::new(&tx);
            repo.delete(org_id, written.id).await?;
            repo.find_by_id(org_id, written.id).await
        })
        .await
        .unwrap();

        assert_eq!(found, None);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn deleting_a_folder_holding_a_sub_folder_is_refused() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "folder-delete-child").await;
        let parent = insert(&pool, &folder(org_id, None, "Parent"))
            .await
            .unwrap();
        insert(&pool, &folder(org_id, Some(parent.id), "Child"))
            .await
            .unwrap();

        let outcome = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowFolderRepository::new(&tx);
            repo.delete(org_id, parent.id).await
        })
        .await;

        assert!(
            matches!(&outcome, Err(CoreError::Conflict(m)) if FolderRefusal::from_conflict(m) == Some(FolderRefusal::NotEmpty))
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn deleting_a_folder_holding_a_workflow_is_refused_even_when_it_is_archived() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "folder-delete-workflow").await;
        let active_home = insert(&pool, &folder(org_id, None, "Active"))
            .await
            .unwrap();
        let archive_home = insert(&pool, &folder(org_id, None, "Archive"))
            .await
            .unwrap();
        seed_workflow_in(&pool, org_id, active_home.id, false).await;
        seed_workflow_in(&pool, org_id, archive_home.id, true).await;

        for home in [&active_home, &archive_home] {
            let outcome = with_tx(&pool, async |tx| {
                let mut repo = PgWorkflowFolderRepository::new(&tx);
                repo.delete(org_id, home.id).await
            })
            .await;

            assert!(
                matches!(&outcome, Err(CoreError::Conflict(m)) if FolderRefusal::from_conflict(m) == Some(FolderRefusal::NotEmpty))
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn deleting_a_missing_or_foreign_folder_is_not_found() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "folder-delete-missing").await;
        let stranger = seed_organization(&pool, "folder-delete-stranger").await;
        let written = insert(&pool, &folder(org_id, None, "Mine")).await.unwrap();

        let missing = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowFolderRepository::new(&tx);
            repo.delete(org_id, generate_uuid_v7()).await
        })
        .await;
        let foreign = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowFolderRepository::new(&tx);
            repo.delete(stranger, written.id).await
        })
        .await;

        assert!(matches!(missing, Err(CoreError::NotFound)));
        assert!(matches!(foreign, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn deleting_an_organization_removes_its_nested_folders_and_filed_workflows() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "folder-org-cascade").await;
        let root = insert(&pool, &folder(org_id, None, "Root")).await.unwrap();
        let child = insert(&pool, &folder(org_id, Some(root.id), "Child"))
            .await
            .unwrap();
        seed_workflow_in(&pool, org_id, child.id, false).await;

        sqlx::query("DELETE FROM organizations WHERE id = $1")
            .bind(org_id.0)
            .execute(&pool)
            .await
            .unwrap();

        let left: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM automation.workflow_folder WHERE org_id = $1")
                .bind(org_id.0)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(left, 0);
    }
}

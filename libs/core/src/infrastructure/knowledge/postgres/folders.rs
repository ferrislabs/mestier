use common::CoreError;
use mestier_macros::repository;

use crate::{
    KnowledgeFolder, KnowledgeFolderId, OrganizationId,
    domain::knowledge::ports::KnowledgeFolderRepository,
    infrastructure::{
        knowledge::postgres::model::KnowledgeFolderRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = KnowledgeFolder, backend = Postgres)]
pub struct PgKnowledgeFolderRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgKnowledgeFolderRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> KnowledgeFolderRepository for PgKnowledgeFolderRepository<'tx> {
    async fn insert(&mut self, folder: &KnowledgeFolder) -> Result<KnowledgeFolder, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            KnowledgeFolderRow,
            r#"
            INSERT INTO knowledge_folders (id, org_id, parent_id, name, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, org_id, parent_id, name, created_at, updated_at
            "#,
            folder.id.0,
            folder.organization_id.0,
            folder.parent_id.map(|id| id.0),
            folder.name,
            folder.created_at,
            folder.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn find_by_id(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeFolderId,
    ) -> Result<Option<KnowledgeFolder>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            KnowledgeFolderRow,
            r#"
            SELECT id, org_id, parent_id, name, created_at, updated_at
            FROM knowledge_folders
            WHERE id = $1 AND org_id = $2
            "#,
            id.0,
            organization_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(KnowledgeFolder::from))
    }

    async fn children(
        &mut self,
        organization_id: OrganizationId,
        parent_id: Option<KnowledgeFolderId>,
    ) -> Result<Vec<KnowledgeFolder>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            KnowledgeFolderRow,
            r#"
            SELECT id, org_id, parent_id, name, created_at, updated_at
            FROM knowledge_folders
            WHERE org_id = $1 AND parent_id IS NOT DISTINCT FROM $2
            ORDER BY lower(name), id
            "#,
            organization_id.0,
            parent_id.map(|id| id.0),
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(KnowledgeFolder::from).collect())
    }

    async fn path_to(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeFolderId,
    ) -> Result<Vec<KnowledgeFolder>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            KnowledgeFolderRow,
            r#"
            WITH RECURSIVE path AS (
                SELECT f.id, f.org_id, f.parent_id, f.name, f.created_at, f.updated_at, 0 AS up
                FROM knowledge_folders f
                WHERE f.id = $1 AND f.org_id = $2
                UNION ALL
                SELECT f.id, f.org_id, f.parent_id, f.name, f.created_at, f.updated_at, p.up + 1
                FROM knowledge_folders f
                JOIN path p ON f.id = p.parent_id AND f.org_id = p.org_id
                WHERE p.up < 64
            )
            SELECT id AS "id!", org_id AS "org_id!", parent_id, name AS "name!",
                   created_at AS "created_at!", updated_at AS "updated_at!"
            FROM path
            ORDER BY up DESC
            "#,
            id.0,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(KnowledgeFolder::from).collect())
    }

    async fn height(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeFolderId,
    ) -> Result<u32, CoreError> {
        let mut tx = self.tx.lock().await;
        let height = sqlx::query_scalar!(
            r#"
            WITH RECURSIVE tree AS (
                SELECT f.id, 0 AS down
                FROM knowledge_folders f
                WHERE f.id = $1 AND f.org_id = $2
                UNION ALL
                SELECT f.id, t.down + 1
                FROM knowledge_folders f
                JOIN tree t ON f.parent_id = t.id
                WHERE t.down < 64
            )
            SELECT COALESCE(MAX(down), 0) AS "height!" FROM tree
            "#,
            id.0,
            organization_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(u32::try_from(height).unwrap_or_default())
    }

    async fn is_empty(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeFolderId,
    ) -> Result<bool, CoreError> {
        let mut tx = self.tx.lock().await;
        let empty = sqlx::query_scalar!(
            r#"
            SELECT NOT EXISTS (
                SELECT 1 FROM knowledge_folders WHERE org_id = $1 AND parent_id = $2
            ) AND NOT EXISTS (
                SELECT 1 FROM knowledge_articles WHERE org_id = $1 AND folder_id = $2
            ) AS "empty!"
            "#,
            organization_id.0,
            id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(empty)
    }

    async fn update(&mut self, folder: &KnowledgeFolder) -> Result<KnowledgeFolder, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            KnowledgeFolderRow,
            r#"
            UPDATE knowledge_folders
            SET name = $3, parent_id = $4, updated_at = $5
            WHERE id = $1 AND org_id = $2
            RETURNING id, org_id, parent_id, name, created_at, updated_at
            "#,
            folder.id.0,
            folder.organization_id.0,
            folder.name,
            folder.parent_id.map(|id| id.0),
            folder.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .ok_or(CoreError::NotFound)?;

        Ok(row.into())
    }

    async fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeFolderId,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            "DELETE FROM knowledge_folders WHERE id = $1 AND org_id = $2",
            id.0,
            organization_id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }
}

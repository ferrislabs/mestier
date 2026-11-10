use common::CoreError;
use mestier_macros::repository;

use crate::{
    KnowledgeArticle, KnowledgeArticleId, KnowledgeFilter, KnowledgeFolderId, KnowledgeFolderScope,
    KnowledgeLink, KnowledgeResource, OrganizationId,
    domain::knowledge::ports::KnowledgeArticleRepository,
    infrastructure::{
        knowledge::postgres::model::{KnowledgeArticleRow, KnowledgeLinkRow},
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = KnowledgeArticle, backend = Postgres)]
pub struct PgKnowledgeArticleRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgKnowledgeArticleRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> KnowledgeArticleRepository for PgKnowledgeArticleRepository<'tx> {
    async fn insert(&mut self, article: &KnowledgeArticle) -> Result<KnowledgeArticle, CoreError> {
        let file = article.file.as_ref();
        let size_bytes = file
            .map(|file| {
                i64::try_from(file.size_bytes)
                    .map_err(|_| CoreError::Internal("size_bytes out of range".into()))
            })
            .transpose()?;

        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            KnowledgeArticleRow,
            r#"
            INSERT INTO knowledge_articles (id, org_id, folder_id, kind, title, body, file_key, mime_type, size_bytes, extracted_text, tags, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id, org_id, folder_id, kind, title, body, file_key, mime_type, size_bytes, extracted_text, tags, created_at, updated_at
            "#,
            article.id.0,
            article.organization_id.0,
            article.folder_id.map(|id| id.0),
            article.kind.as_str(),
            article.title,
            article.body,
            file.map(|file| file.key.as_str()),
            file.map(|file| file.mime_type.as_str()),
            size_bytes,
            article.extracted_text,
            &article.tags,
            article.created_at,
            article.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.try_into()
    }

    async fn find_by_id(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
    ) -> Result<Option<KnowledgeArticle>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            KnowledgeArticleRow,
            r#"
            SELECT id, org_id, folder_id, kind, title, body, file_key, mime_type, size_bytes, extracted_text, tags, created_at, updated_at
            FROM knowledge_articles
            WHERE id = $1 AND org_id = $2
            "#,
            id.0,
            organization_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(KnowledgeArticle::try_from).transpose()
    }

    async fn list(
        &mut self,
        organization_id: OrganizationId,
        filter: &KnowledgeFilter,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<KnowledgeArticle>, u64), CoreError> {
        let limit = i64::try_from(limit).unwrap_or(i64::MAX);
        let offset = i64::try_from(offset).unwrap_or(i64::MAX);
        let query = filter
            .query
            .as_deref()
            .map(str::trim)
            .filter(|q| !q.is_empty());
        let resource_type = filter.resource.map(|r| r.resource_type.as_str());
        let resource_id = filter.resource.map(|r| r.id);
        let (folder_mode, folder_id) = match filter.folder {
            None => (None, None),
            Some(KnowledgeFolderScope::Root) => (Some("root"), None),
            Some(KnowledgeFolderScope::Directly(id)) => (Some("directly"), Some(id.0)),
            Some(KnowledgeFolderScope::Within(id)) => (Some("within"), Some(id.0)),
        };

        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            KnowledgeArticleRow,
            r#"
            SELECT a.id, a.org_id, a.folder_id, a.kind, a.title, a.body, a.file_key, a.mime_type, a.size_bytes,
                   NULL::text AS extracted_text, a.tags, a.created_at, a.updated_at
            FROM knowledge_articles a
            WHERE a.org_id = $1
              AND ($2::text IS NULL OR a.search @@ websearch_to_tsquery('french', $2))
              AND a.tags @> $3::text[]
              AND ($4::text IS NULL OR EXISTS (
                    SELECT 1 FROM knowledge_links l
                    WHERE l.article_id = a.id AND l.resource_type = $4 AND l.resource_id = $5))
              AND CASE $6::text
                    WHEN 'root' THEN a.folder_id IS NULL
                    WHEN 'directly' THEN a.folder_id = $7
                    WHEN 'within' THEN a.folder_id IN (
                        WITH RECURSIVE tree AS (
                            SELECT f.id FROM knowledge_folders f WHERE f.id = $7 AND f.org_id = $1
                            UNION ALL
                            SELECT f.id FROM knowledge_folders f JOIN tree t ON f.parent_id = t.id
                        )
                        SELECT id FROM tree)
                    ELSE TRUE
                  END
            ORDER BY
              CASE WHEN $2::text IS NULL THEN 0 ELSE ts_rank(a.search, websearch_to_tsquery('french', $2)) END DESC,
              a.created_at DESC, a.id
            LIMIT $8 OFFSET $9
            "#,
            organization_id.0,
            query,
            &filter.tags,
            resource_type,
            resource_id,
            folder_mode,
            folder_id,
            limit,
            offset,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let total = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM knowledge_articles a
            WHERE a.org_id = $1
              AND ($2::text IS NULL OR a.search @@ websearch_to_tsquery('french', $2))
              AND a.tags @> $3::text[]
              AND ($4::text IS NULL OR EXISTS (
                    SELECT 1 FROM knowledge_links l
                    WHERE l.article_id = a.id AND l.resource_type = $4 AND l.resource_id = $5))
              AND CASE $6::text
                    WHEN 'root' THEN a.folder_id IS NULL
                    WHEN 'directly' THEN a.folder_id = $7
                    WHEN 'within' THEN a.folder_id IN (
                        WITH RECURSIVE tree AS (
                            SELECT f.id FROM knowledge_folders f WHERE f.id = $7 AND f.org_id = $1
                            UNION ALL
                            SELECT f.id FROM knowledge_folders f JOIN tree t ON f.parent_id = t.id
                        )
                        SELECT id FROM tree)
                    ELSE TRUE
                  END
            "#,
            organization_id.0,
            query,
            &filter.tags,
            resource_type,
            resource_id,
            folder_mode,
            folder_id,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let articles = rows
            .into_iter()
            .map(KnowledgeArticle::try_from)
            .collect::<Result<_, _>>()?;
        Ok((articles, u64::try_from(total).unwrap_or_default()))
    }

    async fn update(&mut self, article: &KnowledgeArticle) -> Result<KnowledgeArticle, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            KnowledgeArticleRow,
            r#"
            UPDATE knowledge_articles
            SET title = $3, body = $4, tags = $5, updated_at = $6
            WHERE id = $1 AND org_id = $2
            RETURNING id, org_id, folder_id, kind, title, body, file_key, mime_type, size_bytes, extracted_text, tags, created_at, updated_at
            "#,
            article.id.0,
            article.organization_id.0,
            article.title,
            article.body,
            &article.tags,
            article.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .ok_or(CoreError::NotFound)?;

        row.try_into()
    }

    async fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            "DELETE FROM knowledge_articles WHERE id = $1 AND org_id = $2",
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

    async fn add_link(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
        resource: KnowledgeResource,
    ) -> Result<KnowledgeLink, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            KnowledgeLinkRow,
            r#"
            INSERT INTO knowledge_links (article_id, org_id, resource_type, resource_id)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (article_id, resource_type, resource_id)
              DO UPDATE SET resource_id = EXCLUDED.resource_id
              WHERE knowledge_links.org_id = EXCLUDED.org_id
            RETURNING article_id, resource_type, resource_id, created_at
            "#,
            id.0,
            organization_id.0,
            resource.resource_type.as_str(),
            resource.id,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .ok_or(CoreError::NotFound)?;

        row.try_into()
    }

    async fn remove_link(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
        resource: KnowledgeResource,
    ) -> Result<bool, CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            DELETE FROM knowledge_links
            WHERE article_id = $1 AND org_id = $2 AND resource_type = $3 AND resource_id = $4
            "#,
            id.0,
            organization_id.0,
            resource.resource_type.as_str(),
            resource.id,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(result.rows_affected() > 0)
    }

    async fn links_of(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
    ) -> Result<Vec<KnowledgeLink>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            KnowledgeLinkRow,
            r#"
            SELECT article_id, resource_type, resource_id, created_at
            FROM knowledge_links
            WHERE article_id = $1 AND org_id = $2
            ORDER BY created_at, resource_type, resource_id
            "#,
            id.0,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(KnowledgeLink::try_from).collect()
    }

    async fn move_to_folder(
        &mut self,
        organization_id: OrganizationId,
        ids: &[KnowledgeArticleId],
        folder_id: Option<KnowledgeFolderId>,
    ) -> Result<u64, CoreError> {
        let ids: Vec<_> = ids.iter().map(|id| id.0).collect();
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE knowledge_articles
            SET folder_id = $3, updated_at = now()
            WHERE org_id = $1 AND id = ANY($2)
            "#,
            organization_id.0,
            &ids,
            folder_id.map(|id| id.0),
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(result.rows_affected())
    }
}

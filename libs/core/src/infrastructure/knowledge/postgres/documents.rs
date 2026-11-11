use chrono::{DateTime, Utc};
use common::{CoreError, UserId};
use mestier_macros::repository;
use uuid::Uuid;

use crate::{
    EditLock, KnowledgeArticleId, KnowledgeResourceType, OrganizationId,
    domain::knowledge::documents::{
        content::DocumentContent,
        lock::LOCK_SECONDS,
        ports::{
            DocumentReferenceRepository, DocumentState, KnowledgeDocumentRepository, LockAttempt,
            ReplaceAttempt, SaveAttempt,
        },
        references::DocumentReference,
    },
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

#[repository(domain = KnowledgeDocument, backend = Postgres)]
pub struct PgKnowledgeDocumentRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgKnowledgeDocumentRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

fn lock_of(holder: Option<Uuid>, expires_at: Option<DateTime<Utc>>) -> Option<EditLock> {
    holder.zip(expires_at).map(|(holder, expires_at)| EditLock {
        holder: UserId(holder),
        expires_at,
    })
}

impl<'tx> KnowledgeDocumentRepository for PgKnowledgeDocumentRepository<'tx> {
    async fn state(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
    ) -> Result<Option<DocumentState>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query!(
            r#"
            SELECT content AS "content!", revision, lock_holder, lock_expires_at
            FROM knowledge_articles
            WHERE id = $1 AND org_id = $2 AND kind = 'document'
            "#,
            id.0,
            organization_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(|row| DocumentState {
            content: row.content,
            revision: row.revision,
            lock: lock_of(row.lock_holder, row.lock_expires_at),
        }))
    }

    async fn acquire_lock(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
        user: UserId,
    ) -> Result<LockAttempt, CoreError> {
        let mut tx = self.tx.lock().await;
        let granted = sqlx::query!(
            r#"
            UPDATE knowledge_articles
            SET lock_holder = $3, lock_expires_at = now() + make_interval(secs => $4::float8)
            WHERE id = $1 AND org_id = $2 AND kind = 'document'
              AND (lock_holder IS NULL OR lock_expires_at IS NULL
                   OR lock_expires_at < now() OR lock_holder = $3)
            RETURNING revision, lock_expires_at AS "lock_expires_at!"
            "#,
            id.0,
            organization_id.0,
            user.0,
            LOCK_SECONDS as f64,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        if let Some(row) = granted {
            return Ok(LockAttempt::Granted {
                lock: EditLock {
                    holder: user,
                    expires_at: row.lock_expires_at,
                },
                revision: row.revision,
            });
        }

        let current = sqlx::query!(
            r#"
            SELECT lock_holder, lock_expires_at
            FROM knowledge_articles
            WHERE id = $1 AND org_id = $2 AND kind = 'document'
            "#,
            id.0,
            organization_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        match current {
            None => Ok(LockAttempt::Missing),
            Some(row) => lock_of(row.lock_holder, row.lock_expires_at)
                .map(LockAttempt::Held)
                .ok_or_else(|| CoreError::Conflict("the lock changed, try again".into())),
        }
    }

    async fn release_lock(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
        user: UserId,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            r#"
            UPDATE knowledge_articles
            SET lock_holder = NULL, lock_expires_at = NULL
            WHERE id = $1 AND org_id = $2 AND kind = 'document' AND lock_holder = $3
            "#,
            id.0,
            organization_id.0,
            user.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn save(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
        user: UserId,
        revision: i32,
        content: &DocumentContent,
        body: &str,
    ) -> Result<SaveAttempt, CoreError> {
        let content = content.to_value();
        let mut tx = self.tx.lock().await;
        let saved = sqlx::query!(
            r#"
            UPDATE knowledge_articles
            SET content = $4, body = $5, revision = revision + 1, updated_at = now()
            WHERE id = $1 AND org_id = $2 AND kind = 'document'
              AND lock_holder = $3 AND lock_expires_at >= now() AND revision = $6
            RETURNING revision, updated_at
            "#,
            id.0,
            organization_id.0,
            user.0,
            content,
            body,
            revision,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        if let Some(row) = saved {
            return Ok(SaveAttempt::Saved {
                revision: row.revision,
                updated_at: row.updated_at,
            });
        }

        let current = sqlx::query!(
            r#"
            SELECT coalesce(lock_holder = $3 AND lock_expires_at >= now(), false) AS "holds!"
            FROM knowledge_articles
            WHERE id = $1 AND org_id = $2 AND kind = 'document'
            "#,
            id.0,
            organization_id.0,
            user.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(match current {
            None => SaveAttempt::Missing,
            Some(row) if row.holds => SaveAttempt::RevisionConflict,
            Some(_) => SaveAttempt::LockLost,
        })
    }

    async fn replace(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
        user: UserId,
        content: &DocumentContent,
        body: &str,
    ) -> Result<ReplaceAttempt, CoreError> {
        let content = content.to_value();
        let mut tx = self.tx.lock().await;
        let replaced = sqlx::query!(
            r#"
            UPDATE knowledge_articles
            SET content = $4, body = $5, revision = revision + 1, updated_at = now()
            WHERE id = $1 AND org_id = $2 AND kind = 'document'
              AND (lock_holder IS NULL OR lock_expires_at IS NULL
                   OR lock_expires_at < now() OR lock_holder = $3)
            RETURNING revision, updated_at
            "#,
            id.0,
            organization_id.0,
            user.0,
            content,
            body,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        if let Some(row) = replaced {
            return Ok(ReplaceAttempt::Replaced {
                revision: row.revision,
                updated_at: row.updated_at,
            });
        }

        let current = sqlx::query!(
            r#"
            SELECT lock_holder, lock_expires_at
            FROM knowledge_articles
            WHERE id = $1 AND org_id = $2 AND kind = 'document'
            "#,
            id.0,
            organization_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        match current {
            None => Ok(ReplaceAttempt::Missing),
            Some(row) => lock_of(row.lock_holder, row.lock_expires_at)
                .map(ReplaceAttempt::Held)
                .ok_or_else(|| CoreError::Conflict("the lock changed, try again".into())),
        }
    }
}

#[repository(domain = DocumentReference, backend = Postgres)]
pub struct PgDocumentReferenceRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgDocumentReferenceRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> DocumentReferenceRepository for PgDocumentReferenceRepository<'tx> {
    async fn exists(
        &mut self,
        organization_id: OrganizationId,
        reference: DocumentReference,
    ) -> Result<bool, CoreError> {
        let org = organization_id.0;
        let mut tx = self.tx.lock().await;
        let found = match reference {
            DocumentReference::DriveFile(id) => {
                sqlx::query_scalar!(
                    r#"SELECT EXISTS(SELECT 1 FROM knowledge_articles WHERE id = $1 AND org_id = $2 AND kind = 'file') AS "found!""#,
                    id.0,
                    org,
                )
                .fetch_one(&mut ***tx)
                .await
            }
            DocumentReference::Photo(id) => {
                sqlx::query_scalar!(
                    r#"SELECT EXISTS(SELECT 1 FROM library_photos WHERE id = $1 AND org_id = $2) AS "found!""#,
                    id,
                    org,
                )
                .fetch_one(&mut ***tx)
                .await
            }
            DocumentReference::TableView { table_id, view_id } => {
                sqlx::query_scalar!(
                    r#"
                    SELECT EXISTS(
                        SELECT 1 FROM knowledge_table_views
                        WHERE id = $1 AND table_id = $2 AND org_id = $3
                    ) AS "found!"
                    "#,
                    view_id.0,
                    table_id.0,
                    org,
                )
                .fetch_one(&mut ***tx)
                .await
            }
            DocumentReference::Resource(resource) => match resource.resource_type {
                KnowledgeResourceType::Customer => {
                    sqlx::query_scalar!(
                        r#"SELECT EXISTS(SELECT 1 FROM customers WHERE id = $1 AND org_id = $2) AS "found!""#,
                        resource.id,
                        org,
                    )
                    .fetch_one(&mut ***tx)
                    .await
                }
                KnowledgeResourceType::Project => {
                    sqlx::query_scalar!(
                        r#"SELECT EXISTS(SELECT 1 FROM projects WHERE id = $1 AND org_id = $2) AS "found!""#,
                        resource.id,
                        org,
                    )
                    .fetch_one(&mut ***tx)
                    .await
                }
                KnowledgeResourceType::Quote => {
                    sqlx::query_scalar!(
                        r#"SELECT EXISTS(SELECT 1 FROM quotes WHERE id = $1 AND org_id = $2) AS "found!""#,
                        resource.id,
                        org,
                    )
                    .fetch_one(&mut ***tx)
                    .await
                }
            },
        };
        found.map_err(map_sqlx_error)
    }
}

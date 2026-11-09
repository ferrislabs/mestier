use common::CoreError;
use mestier_macros::repository;

use crate::{
    InsertOutcome, LibraryPhoto, LibraryPhotoId, OrganizationId, PhotoFilter,
    domain::photo_library::ports::LibraryPhotoRepository,
    infrastructure::{
        photo_library::postgres::model::LibraryPhotoRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = LibraryPhoto, backend = Postgres)]
pub struct PgLibraryPhotoRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgLibraryPhotoRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

fn dimension(value: u32, column: &str) -> Result<i32, CoreError> {
    i32::try_from(value).map_err(|_| CoreError::Internal(format!("{column} out of range")))
}

fn contains_pattern(query: &str) -> String {
    let mut pattern = String::with_capacity(query.len() + 2);
    pattern.push('%');
    for character in query.chars() {
        if matches!(character, '\\' | '%' | '_') {
            pattern.push('\\');
        }
        pattern.push(character);
    }
    pattern.push('%');
    pattern
}

impl<'tx> LibraryPhotoRepository for PgLibraryPhotoRepository<'tx> {
    async fn insert_if_absent(&mut self, photo: &LibraryPhoto) -> Result<InsertOutcome, CoreError> {
        let width = dimension(photo.width, "width")?;
        let height = dimension(photo.height, "height")?;
        let size_bytes = dimension(photo.size_bytes, "size_bytes")?;

        let inserted = {
            let mut tx = self.tx.lock().await;
            sqlx::query_as!(
                LibraryPhotoRow,
                r#"
                INSERT INTO library_photos (id, org_id, file_key, thumbnail_key, mime_type, width, height, size_bytes, content_hash, caption, tags, project_id, taken_at, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
                ON CONFLICT (org_id, content_hash) DO NOTHING
                RETURNING id, org_id, file_key, thumbnail_key, mime_type, width, height, size_bytes, content_hash, caption, tags, project_id, taken_at, created_at, updated_at
                "#,
                photo.id.0,
                photo.organization_id.0,
                photo.file_key,
                photo.thumbnail_key,
                photo.mime_type,
                width,
                height,
                size_bytes,
                photo.content_hash,
                photo.caption,
                &photo.tags,
                photo.project_id.map(|id| id.0),
                photo.taken_at,
                photo.created_at,
                photo.updated_at,
            )
            .fetch_optional(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
        };

        if let Some(row) = inserted {
            return Ok(InsertOutcome::Inserted(row.try_into()?));
        }
        self.find_by_hash(photo.organization_id, &photo.content_hash)
            .await?
            .map(InsertOutcome::Existing)
            .ok_or(CoreError::NotFound)
    }

    async fn find_by_id(
        &mut self,
        organization_id: OrganizationId,
        id: LibraryPhotoId,
    ) -> Result<Option<LibraryPhoto>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            LibraryPhotoRow,
            r#"
            SELECT id, org_id, file_key, thumbnail_key, mime_type, width, height, size_bytes, content_hash, caption, tags, project_id, taken_at, created_at, updated_at
            FROM library_photos
            WHERE id = $1 AND org_id = $2
            "#,
            id.0,
            organization_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(LibraryPhoto::try_from).transpose()
    }

    async fn find_by_hash(
        &mut self,
        organization_id: OrganizationId,
        content_hash: &str,
    ) -> Result<Option<LibraryPhoto>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            LibraryPhotoRow,
            r#"
            SELECT id, org_id, file_key, thumbnail_key, mime_type, width, height, size_bytes, content_hash, caption, tags, project_id, taken_at, created_at, updated_at
            FROM library_photos
            WHERE org_id = $1 AND content_hash = $2
            "#,
            organization_id.0,
            content_hash,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(LibraryPhoto::try_from).transpose()
    }

    async fn list(
        &mut self,
        organization_id: OrganizationId,
        filter: &PhotoFilter,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<LibraryPhoto>, u64), CoreError> {
        let limit = i64::try_from(limit).unwrap_or(i64::MAX);
        let offset = i64::try_from(offset).unwrap_or(i64::MAX);
        let pattern = filter.query.as_deref().map(contains_pattern);
        let project_id = filter.project_id.map(|id| id.0);

        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            LibraryPhotoRow,
            r#"
            SELECT id, org_id, file_key, thumbnail_key, mime_type, width, height, size_bytes, content_hash, caption, tags, project_id, taken_at, created_at, updated_at
            FROM library_photos
            WHERE org_id = $1
              AND ($2::text IS NULL
                   OR caption ILIKE $2
                   OR EXISTS (SELECT 1 FROM unnest(tags) AS tag WHERE tag ILIKE $2))
              AND tags @> $3::text[]
              AND ($4::uuid IS NULL OR project_id = $4)
              AND (NOT $5::boolean OR (caption IS NULL AND cardinality(tags) = 0))
            ORDER BY created_at DESC, id
            LIMIT $6 OFFSET $7
            "#,
            organization_id.0,
            pattern,
            &filter.tags,
            project_id,
            filter.to_sort,
            limit,
            offset,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let total = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM library_photos
            WHERE org_id = $1
              AND ($2::text IS NULL
                   OR caption ILIKE $2
                   OR EXISTS (SELECT 1 FROM unnest(tags) AS tag WHERE tag ILIKE $2))
              AND tags @> $3::text[]
              AND ($4::uuid IS NULL OR project_id = $4)
              AND (NOT $5::boolean OR (caption IS NULL AND cardinality(tags) = 0))
            "#,
            organization_id.0,
            pattern,
            &filter.tags,
            project_id,
            filter.to_sort,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let photos = rows
            .into_iter()
            .map(LibraryPhoto::try_from)
            .collect::<Result<_, _>>()?;
        Ok((photos, u64::try_from(total).unwrap_or_default()))
    }

    async fn update(&mut self, photo: &LibraryPhoto) -> Result<LibraryPhoto, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            LibraryPhotoRow,
            r#"
            UPDATE library_photos
            SET caption = $3, tags = $4, project_id = $5, updated_at = $6
            WHERE id = $1 AND org_id = $2
            RETURNING id, org_id, file_key, thumbnail_key, mime_type, width, height, size_bytes, content_hash, caption, tags, project_id, taken_at, created_at, updated_at
            "#,
            photo.id.0,
            photo.organization_id.0,
            photo.caption,
            &photo.tags,
            photo.project_id.map(|id| id.0),
            photo.updated_at,
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
        id: LibraryPhotoId,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            "DELETE FROM library_photos WHERE id = $1 AND org_id = $2",
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

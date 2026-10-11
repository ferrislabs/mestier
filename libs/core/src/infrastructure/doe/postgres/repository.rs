use common::CoreError;
use mestier_macros::repository;

use crate::{
    Doe, DoeId, DoeSummary, OrganizationId, ProjectId,
    domain::doe::ports::DoeRepository,
    infrastructure::{
        doe::postgres::model::{DoeRow, DoeSummaryRow},
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = Doe, backend = Postgres)]
pub struct PgDoeRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgDoeRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

fn to_json<T: serde::Serialize>(value: &T) -> Result<serde_json::Value, CoreError> {
    serde_json::to_value(value)
        .map_err(|e| CoreError::Internal(format!("cannot serialize DOE content: {e}")))
}

impl<'tx> DoeRepository for PgDoeRepository<'tx> {
    async fn insert(&mut self, doe: &Doe) -> Result<Doe, CoreError> {
        let parties = to_json(&doe.parties)?;
        let sections = to_json(&doe.sections)?;
        let revision = doe.revision.as_str();

        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DoeRow,
            r#"
            INSERT INTO does (id, org_id, project_id, model_id, title, place, lot, issued_on, author, revision, cover_photo_key, parties, sections, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING id, org_id, project_id, model_id, title, place, lot, issued_on, author, revision, cover_photo_key, parties, sections, created_at, updated_at
            "#,
            doe.id.0,
            doe.organization_id.0,
            doe.project_id.0,
            doe.model_id.map(|id| id.0),
            doe.title,
            doe.place,
            doe.lot,
            doe.issued_on,
            doe.author,
            revision,
            doe.cover_photo_key,
            parties,
            sections,
            doe.created_at,
            doe.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.try_into()
    }

    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<DoeSummary>, u64), CoreError> {
        let limit = i64::try_from(limit).unwrap_or(i64::MAX);
        let offset = i64::try_from(offset).unwrap_or(i64::MAX);
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            DoeSummaryRow,
            r#"
            SELECT d.id, d.project_id, p.name AS project_name, d.title, d.revision, d.issued_on, d.updated_at
            FROM does d
            JOIN projects p ON p.id = d.project_id AND p.org_id = d.org_id
            WHERE d.org_id = $1
            ORDER BY d.updated_at DESC, d.id
            LIMIT $2 OFFSET $3
            "#,
            organization_id.0,
            limit,
            offset,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        let total = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM does WHERE org_id = $1"#,
            organization_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let summaries = rows
            .into_iter()
            .map(DoeSummary::try_from)
            .collect::<Result<_, _>>()?;
        Ok((summaries, u64::try_from(total).unwrap_or_default()))
    }

    async fn find_by_id(&mut self, id: DoeId) -> Result<Option<Doe>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DoeRow,
            r#"
            SELECT id, org_id, project_id, model_id, title, place, lot, issued_on, author, revision, cover_photo_key, parties, sections, created_at, updated_at
            FROM does
            WHERE id = $1
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Doe::try_from).transpose()
    }

    async fn find_by_project(&mut self, project_id: ProjectId) -> Result<Option<Doe>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DoeRow,
            r#"
            SELECT id, org_id, project_id, model_id, title, place, lot, issued_on, author, revision, cover_photo_key, parties, sections, created_at, updated_at
            FROM does
            WHERE project_id = $1
            "#,
            project_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Doe::try_from).transpose()
    }

    async fn update(&mut self, doe: &Doe) -> Result<Doe, CoreError> {
        let parties = to_json(&doe.parties)?;
        let sections = to_json(&doe.sections)?;
        let revision = doe.revision.as_str();

        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DoeRow,
            r#"
            UPDATE does
            SET title = $2, place = $3, lot = $4, issued_on = $5, author = $6, revision = $7,
                cover_photo_key = $8, parties = $9, sections = $10, updated_at = $11
            WHERE id = $1
            RETURNING id, org_id, project_id, model_id, title, place, lot, issued_on, author, revision, cover_photo_key, parties, sections, created_at, updated_at
            "#,
            doe.id.0,
            doe.title,
            doe.place,
            doe.lot,
            doe.issued_on,
            doe.author,
            revision,
            doe.cover_photo_key,
            parties,
            sections,
            doe.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.try_into()
    }
}

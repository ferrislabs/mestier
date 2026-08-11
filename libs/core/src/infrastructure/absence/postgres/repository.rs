use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    Absence, AbsenceId, OrganizationId,
    domain::absence::ports::AbsenceRepository,
    infrastructure::{
        absence::postgres::model::AbsenceRow, postgres::SharedTx, postgres::error::map_sqlx_error,
    },
};

#[repository(domain = Absence, backend = Postgres)]
pub struct PgAbsenceRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgAbsenceRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> AbsenceRepository for PgAbsenceRepository<'tx> {
    async fn insert(&mut self, absence: &Absence) -> Result<Absence, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            AbsenceRow,
            r#"
            INSERT INTO absences (id, org_id, member_id, kind, starts_at, ends_at, all_day, note, deleted_at, created_at, updated_at)
            VALUES ($1, $2, $3, CAST($4 AS text)::absence_kind, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, org_id, member_id, kind::text AS "kind!", starts_at, ends_at, all_day, note, deleted_at, created_at, updated_at
            "#,
            absence.id.0,
            absence.organization_id.0,
            absence.member_id.0,
            absence.kind.as_str(),
            absence.starts_at,
            absence.ends_at,
            absence.all_day,
            absence.note,
            absence.deleted_at,
            absence.created_at,
            absence.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.into_employee_absence()
    }

    async fn find_by_id(&mut self, id: AbsenceId) -> Result<Option<Absence>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            AbsenceRow,
            r#"
            SELECT id, org_id, member_id, kind::text AS "kind!", starts_at, ends_at, all_day, note, deleted_at, created_at, updated_at
            FROM absences
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(AbsenceRow::into_employee_absence).transpose()
    }

    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Absence>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            AbsenceRow,
            r#"
            SELECT id, org_id, member_id, kind::text AS "kind!", starts_at, ends_at, all_day, note, deleted_at, created_at, updated_at
            FROM absences
            WHERE org_id = $1 AND deleted_at IS NULL
            ORDER BY starts_at ASC, id ASC
            LIMIT $2 OFFSET $3
            "#,
            organization_id.0,
            limit as i64,
            offset as i64,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM absences WHERE org_id = $1 AND deleted_at IS NULL"#,
            organization_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let mut absences = Vec::with_capacity(rows.len());
        for row in rows {
            absences.push(row.into_employee_absence()?);
        }

        Ok((absences, total as u64))
    }

    async fn update(&mut self, absence: &Absence) -> Result<Absence, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            AbsenceRow,
            r#"
            UPDATE absences
            SET kind = CAST($2 AS text)::absence_kind,
                starts_at = $3,
                ends_at = $4,
                all_day = $5,
                note = $6,
                updated_at = $7
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, org_id, member_id, kind::text AS "kind!", starts_at, ends_at, all_day, note, deleted_at, created_at, updated_at
            "#,
            absence.id.0,
            absence.kind.as_str(),
            absence.starts_at,
            absence.ends_at,
            absence.all_day,
            absence.note,
            absence.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let row = row.ok_or(CoreError::NotFound)?;
        row.into_employee_absence()
    }

    async fn soft_delete(
        &mut self,
        id: AbsenceId,
        deleted_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE absences
            SET deleted_at = $2, updated_at = $2
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
            deleted_at,
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

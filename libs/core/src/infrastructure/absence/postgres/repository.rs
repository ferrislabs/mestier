use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    Absence, AbsenceId, MemberId, OrganizationId,
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
            INSERT INTO absences (id, org_id, member_id, kind, type_id, starts_at, ends_at, all_day, start_date, start_part, end_date, end_part, status, note, proof_file_key, created_by_member_id, decided_by_member_id, decided_at, decision_comment, deleted_at, created_at, updated_at)
            VALUES ($1, $2, $3, CAST($4 AS text)::absence_kind, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)
            RETURNING id, org_id, member_id, kind::text AS "kind!", type_id, starts_at, ends_at, all_day, start_date, start_part, end_date, end_part, status, note, proof_file_key, created_by_member_id, decided_by_member_id, decided_at, decision_comment, deleted_at, created_at, updated_at
            "#,
            absence.id.0,
            absence.organization_id.0,
            absence.member_id.0,
            absence.kind.as_str(),
            absence.type_id.0,
            absence.starts_at,
            absence.ends_at,
            absence.all_day,
            absence.start_date,
            absence.start_part.as_str(),
            absence.end_date,
            absence.end_part.as_str(),
            absence.status.as_str(),
            absence.note,
            absence.proof_file_key,
            absence.created_by_member_id.map(|id| id.0),
            absence.decided_by_member_id.map(|id| id.0),
            absence.decided_at,
            absence.decision_comment,
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
            SELECT id, org_id, member_id, kind::text AS "kind!", type_id, starts_at, ends_at, all_day, start_date, start_part, end_date, end_part, status, note, proof_file_key, created_by_member_id, decided_by_member_id, decided_at, decision_comment, deleted_at, created_at, updated_at
            FROM absences
            WHERE id = $1 AND deleted_at IS NULL
            FOR UPDATE
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
            SELECT id, org_id, member_id, kind::text AS "kind!", type_id, starts_at, ends_at, all_day, start_date, start_part, end_date, end_part, status, note, proof_file_key, created_by_member_id, decided_by_member_id, decided_at, decision_comment, deleted_at, created_at, updated_at
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

    async fn list_by_member(
        &mut self,
        organization_id: OrganizationId,
        member_id: MemberId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Absence>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            AbsenceRow,
            r#"
            SELECT id, org_id, member_id, kind::text AS "kind!", type_id, starts_at, ends_at, all_day, start_date, start_part, end_date, end_part, status, note, proof_file_key, created_by_member_id, decided_by_member_id, decided_at, decision_comment, deleted_at, created_at, updated_at
            FROM absences
            WHERE org_id = $1 AND member_id = $2 AND deleted_at IS NULL
            ORDER BY starts_at ASC, id ASC
            LIMIT $3 OFFSET $4
            "#,
            organization_id.0,
            member_id.0,
            limit as i64,
            offset as i64,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM absences WHERE org_id = $1 AND member_id = $2 AND deleted_at IS NULL"#,
            organization_id.0,
            member_id.0,
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

    async fn list_overlapping(
        &mut self,
        organization_id: OrganizationId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<Absence>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            AbsenceRow,
            r#"
            SELECT id, org_id, member_id, kind::text AS "kind!", type_id, starts_at, ends_at, all_day, start_date, start_part, end_date, end_part, status, note, proof_file_key, created_by_member_id, decided_by_member_id, decided_at, decision_comment, deleted_at, created_at, updated_at
            FROM absences
            WHERE org_id = $1 AND deleted_at IS NULL
              AND start_date <= $3 AND end_date >= $2
            ORDER BY start_date ASC, id ASC
            "#,
            organization_id.0,
            from,
            to,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter()
            .map(AbsenceRow::into_employee_absence)
            .collect()
    }

    async fn update(&mut self, absence: &Absence) -> Result<Absence, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            AbsenceRow,
            r#"
            UPDATE absences
            SET kind = CAST($2 AS text)::absence_kind,
                type_id = $3,
                starts_at = $4,
                ends_at = $5,
                all_day = $6,
                start_date = $7,
                start_part = $8,
                end_date = $9,
                end_part = $10,
                status = $11,
                note = $12,
                proof_file_key = $13,
                decided_by_member_id = $14,
                decided_at = $15,
                decision_comment = $16,
                updated_at = $17
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, org_id, member_id, kind::text AS "kind!", type_id, starts_at, ends_at, all_day, start_date, start_part, end_date, end_part, status, note, proof_file_key, created_by_member_id, decided_by_member_id, decided_at, decision_comment, deleted_at, created_at, updated_at
            "#,
            absence.id.0,
            absence.kind.as_str(),
            absence.type_id.0,
            absence.starts_at,
            absence.ends_at,
            absence.all_day,
            absence.start_date,
            absence.start_part.as_str(),
            absence.end_date,
            absence.end_part.as_str(),
            absence.status.as_str(),
            absence.note,
            absence.proof_file_key,
            absence.decided_by_member_id.map(|id| id.0),
            absence.decided_at,
            absence.decision_comment,
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

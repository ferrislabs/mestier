use common::CoreError;
use mestier_macros::repository;

use crate::{
    AssignmentReport, AssignmentReportId, AssignmentReportResolution, MemberId, OrganizationId,
    TaskAssignmentId, TaskId,
    domain::assignment_report::ports::{AssignmentContext, AssignmentReportRepository},
    infrastructure::{
        assignment_report::postgres::model::AssignmentReportRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = AssignmentReport, backend = Postgres)]
pub struct PgAssignmentReportRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgAssignmentReportRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> AssignmentReportRepository for PgAssignmentReportRepository<'tx> {
    async fn find_assignment_context(
        &mut self,
        task_assignment_id: TaskAssignmentId,
    ) -> Result<Option<AssignmentContext>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query!(
            r#"
            SELECT org_id, task_id, member_id
            FROM task_assignments
            WHERE id = $1
            "#,
            task_assignment_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(|r| AssignmentContext {
            organization_id: OrganizationId(r.org_id),
            task_id: TaskId(r.task_id),
            member_id: MemberId(r.member_id),
        }))
    }

    async fn insert(&mut self, report: &AssignmentReport) -> Result<AssignmentReport, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            AssignmentReportRow,
            r#"
            INSERT INTO assignment_reports (
                id, org_id, task_assignment_id, reported_minutes, comment, reported_by,
                resolution, resolved_by, resolved_at, resolution_note, created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6,
                CAST($7 AS text)::assignment_report_resolution, $8, $9, $10, $11, $12
            )
            RETURNING
                id, org_id, task_assignment_id, reported_minutes, comment, reported_by,
                resolution::text AS "resolution!", resolved_by, resolved_at, resolution_note,
                created_at, updated_at
            "#,
            report.id.0,
            report.organization_id.0,
            report.task_assignment_id.0,
            report.reported_minutes as i32,
            report.comment,
            report.reported_by.0,
            report.resolution.as_str(),
            report.resolved_by.map(|id| id.0),
            report.resolved_at,
            report.resolution_note,
            report.created_at,
            report.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.try_into()
    }

    async fn find_by_id(
        &mut self,
        id: AssignmentReportId,
    ) -> Result<Option<AssignmentReport>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            AssignmentReportRow,
            r#"
            SELECT
                id, org_id, task_assignment_id, reported_minutes, comment, reported_by,
                resolution::text AS "resolution!", resolved_by, resolved_at, resolution_note,
                created_at, updated_at
            FROM assignment_reports
            WHERE id = $1
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(TryInto::try_into).transpose()
    }

    async fn list_by_reporter(
        &mut self,
        organization_id: OrganizationId,
        reported_by: MemberId,
        resolution: Option<AssignmentReportResolution>,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<AssignmentReport>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let resolution_filter = resolution.map(AssignmentReportResolution::as_str);

        let rows = sqlx::query_as!(
            AssignmentReportRow,
            r#"
            SELECT
                id, org_id, task_assignment_id, reported_minutes, comment, reported_by,
                resolution::text AS "resolution!", resolved_by, resolved_at, resolution_note,
                created_at, updated_at
            FROM assignment_reports
            WHERE org_id = $1
              AND reported_by = $2
              AND ($3::text IS NULL OR resolution::text = $3)
            ORDER BY created_at DESC, id DESC
            LIMIT $4 OFFSET $5
            "#,
            organization_id.0,
            reported_by.0,
            resolution_filter,
            limit as i64,
            offset as i64,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let total: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM assignment_reports
            WHERE org_id = $1
              AND reported_by = $2
              AND ($3::text IS NULL OR resolution::text = $3)
            "#,
            organization_id.0,
            reported_by.0,
            resolution_filter,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let items = rows
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        Ok((items, total as u64))
    }

    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        resolution: Option<AssignmentReportResolution>,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<AssignmentReport>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let resolution_filter = resolution.map(AssignmentReportResolution::as_str);

        let rows = sqlx::query_as!(
            AssignmentReportRow,
            r#"
            SELECT
                id, org_id, task_assignment_id, reported_minutes, comment, reported_by,
                resolution::text AS "resolution!", resolved_by, resolved_at, resolution_note,
                created_at, updated_at
            FROM assignment_reports
            WHERE org_id = $1
              AND ($2::text IS NULL OR resolution::text = $2)
            ORDER BY created_at DESC, id DESC
            LIMIT $3 OFFSET $4
            "#,
            organization_id.0,
            resolution_filter,
            limit as i64,
            offset as i64,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let total: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM assignment_reports
            WHERE org_id = $1
              AND ($2::text IS NULL OR resolution::text = $2)
            "#,
            organization_id.0,
            resolution_filter,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let items = rows
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        Ok((items, total as u64))
    }

    async fn update(&mut self, report: &AssignmentReport) -> Result<AssignmentReport, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            AssignmentReportRow,
            r#"
            UPDATE assignment_reports
            SET reported_minutes = $2,
                comment = $3,
                resolution = CAST($4 AS text)::assignment_report_resolution,
                resolved_by = $5,
                resolved_at = $6,
                resolution_note = $7,
                updated_at = $8
            WHERE id = $1
            RETURNING
                id, org_id, task_assignment_id, reported_minutes, comment, reported_by,
                resolution::text AS "resolution!", resolved_by, resolved_at, resolution_note,
                created_at, updated_at
            "#,
            report.id.0,
            report.reported_minutes as i32,
            report.comment,
            report.resolution.as_str(),
            report.resolved_by.map(|id| id.0),
            report.resolved_at,
            report.resolution_note,
            report.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(TryInto::try_into)
            .transpose()?
            .ok_or(CoreError::NotFound)
    }

    async fn delete(&mut self, id: AssignmentReportId) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!("DELETE FROM assignment_reports WHERE id = $1", id.0)
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }
}

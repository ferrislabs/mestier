use common::CoreError;
use mestier_macros::repository;

use crate::{
    EmployeeId, OrganizationId, TimeEntry, TimeEntryId,
    domain::time_entry::ports::TimeEntryRepository,
    infrastructure::{
        postgres::SharedTx, postgres::error::map_sqlx_error,
        time_entry::postgres::model::TimeEntryRow,
    },
};

#[repository(domain = TimeEntry, backend = Postgres)]
pub struct PgTimeEntryRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgTimeEntryRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> TimeEntryRepository for PgTimeEntryRepository<'tx> {
    async fn insert(&mut self, time_entry: &TimeEntry) -> Result<TimeEntry, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TimeEntryRow,
            r#"
            INSERT INTO time_entries (
                id, org_id, work_order_id, employee_id, started_at, ended_at,
                photos_before, photos_during, photos_after, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING
                id, org_id, work_order_id, employee_id, started_at, ended_at,
                photos_before, photos_during, photos_after, created_at, updated_at
            "#,
            time_entry.id.0,
            time_entry.organization_id.0,
            time_entry.work_order_id.0,
            time_entry.employee_id.0,
            time_entry.started_at,
            time_entry.ended_at,
            &time_entry.photos_before,
            &time_entry.photos_during,
            &time_entry.photos_after,
            time_entry.created_at,
            time_entry.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into_time_entry())
    }

    async fn find_by_id(&mut self, id: TimeEntryId) -> Result<Option<TimeEntry>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TimeEntryRow,
            r#"
            SELECT
                id, org_id, work_order_id, employee_id, started_at, ended_at,
                photos_before, photos_during, photos_after, created_at, updated_at
            FROM time_entries
            WHERE id = $1
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(TimeEntryRow::into_time_entry))
    }

    async fn find_active_by_employee(
        &mut self,
        organization_id: OrganizationId,
        employee_id: EmployeeId,
    ) -> Result<Option<TimeEntry>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TimeEntryRow,
            r#"
            SELECT
                id, org_id, work_order_id, employee_id, started_at, ended_at,
                photos_before, photos_during, photos_after, created_at, updated_at
            FROM time_entries
            WHERE org_id = $1 AND employee_id = $2 AND ended_at IS NULL
            "#,
            organization_id.0,
            employee_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(TimeEntryRow::into_time_entry))
    }

    async fn update(&mut self, time_entry: &TimeEntry) -> Result<TimeEntry, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TimeEntryRow,
            r#"
            UPDATE time_entries
            SET ended_at = $2,
                photos_before = $3,
                photos_during = $4,
                photos_after = $5,
                updated_at = $6
            WHERE id = $1
            RETURNING
                id, org_id, work_order_id, employee_id, started_at, ended_at,
                photos_before, photos_during, photos_after, created_at, updated_at
            "#,
            time_entry.id.0,
            time_entry.ended_at,
            &time_entry.photos_before,
            &time_entry.photos_during,
            &time_entry.photos_after,
            time_entry.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let row = row.ok_or(CoreError::NotFound)?;
        Ok(row.into_time_entry())
    }
}

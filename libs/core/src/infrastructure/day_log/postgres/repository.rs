use chrono::NaiveDate;
use common::CoreError;
use mestier_macros::repository;

use crate::{
    DayLog, DayLogId, EmployeeId, OrganizationId,
    domain::day_log::ports::DayLogRepository,
    infrastructure::{
        day_log::postgres::model::DayLogRow, postgres::SharedTx, postgres::error::map_sqlx_error,
    },
};

#[repository(domain = DayLog, backend = Postgres)]
pub struct PgDayLogRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgDayLogRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> DayLogRepository for PgDayLogRepository<'tx> {
    async fn insert(&mut self, day_log: &DayLog) -> Result<DayLog, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DayLogRow,
            r#"
            INSERT INTO day_logs (id, org_id, employee_id, work_date, ended_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, org_id, employee_id, work_date, ended_at, created_at
            "#,
            day_log.id.0,
            day_log.organization_id.0,
            day_log.employee_id.0,
            day_log.work_date,
            day_log.ended_at,
            day_log.created_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into_day_log())
    }

    async fn find_by_id(&mut self, id: DayLogId) -> Result<Option<DayLog>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DayLogRow,
            r#"
            SELECT id, org_id, employee_id, work_date, ended_at, created_at
            FROM day_logs
            WHERE id = $1
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(DayLogRow::into_day_log))
    }

    async fn find_by_employee_and_date(
        &mut self,
        organization_id: OrganizationId,
        employee_id: EmployeeId,
        work_date: NaiveDate,
    ) -> Result<Option<DayLog>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DayLogRow,
            r#"
            SELECT id, org_id, employee_id, work_date, ended_at, created_at
            FROM day_logs
            WHERE org_id = $1 AND employee_id = $2 AND work_date = $3
            "#,
            organization_id.0,
            employee_id.0,
            work_date,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(DayLogRow::into_day_log))
    }
}

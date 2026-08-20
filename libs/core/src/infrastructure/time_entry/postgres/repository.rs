use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    DayLog, EmployeeId, OrganizationId, TimeEntry, TimeEntryId, TimeEntryPhoto,
    domain::time_entry::ports::{DayLogRepository, TimeEntryRepository},
    infrastructure::{
        postgres::SharedTx,
        postgres::error::map_sqlx_error,
        time_entry::postgres::model::{DayLogRow, TimeEntryPhotoRow, TimeEntryRow},
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
    async fn insert(&mut self, entry: &TimeEntry) -> Result<TimeEntry, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TimeEntryRow,
            r#"
            INSERT INTO time_entries (id, org_id, task_id, employee_id, started_at, ended_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, org_id, task_id, employee_id, started_at, ended_at, closed_after_the_fact, created_at, updated_at
            "#,
            entry.id.0,
            entry.organization_id.0,
            entry.task_id.0,
            entry.employee_id.0,
            entry.started_at,
            entry.ended_at,
            entry.created_at,
            entry.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        // A freshly inserted entry has no photos yet, so no second query.
        Ok(row.into_time_entry(vec![]))
    }

    async fn find_by_id(&mut self, id: TimeEntryId) -> Result<Option<TimeEntry>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TimeEntryRow,
            r#"
            SELECT id, org_id, task_id, employee_id, started_at, ended_at, closed_after_the_fact, created_at, updated_at
            FROM time_entries
            WHERE id = $1
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let Some(row) = row else {
            return Ok(None);
        };
        let photos = load_photos(&mut tx, id).await?;

        Ok(Some(row.into_time_entry(photos)))
    }

    /// Relies on the partial unique index for the "at most one" guarantee:
    /// `fetch_optional` would error on a second row, which is the right
    /// outcome since the index makes that state impossible.
    async fn find_running_for_employee(
        &mut self,
        employee_id: EmployeeId,
    ) -> Result<Option<TimeEntry>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TimeEntryRow,
            r#"
            SELECT id, org_id, task_id, employee_id, started_at, ended_at, closed_after_the_fact, created_at, updated_at
            FROM time_entries
            WHERE employee_id = $1 AND ended_at IS NULL
            "#,
            employee_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let Some(row) = row else {
            return Ok(None);
        };
        let photos = load_photos(&mut tx, TimeEntryId(row.id)).await?;

        Ok(Some(row.into_time_entry(photos)))
    }

    /// Entries overlapping the day, keyed on `started_at`: an entry begun at
    /// 23:00 and closed after midnight belongs to the day it started, which is
    /// the day the employee will be looking at.
    async fn list_for_employee_on(
        &mut self,
        employee_id: EmployeeId,
        work_date: NaiveDate,
    ) -> Result<Vec<TimeEntry>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            TimeEntryRow,
            r#"
            SELECT id, org_id, task_id, employee_id, started_at, ended_at, closed_after_the_fact, created_at, updated_at
            FROM time_entries
            WHERE employee_id = $1 AND started_at::date = $2
            ORDER BY started_at
            "#,
            employee_id.0,
            work_date,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let mut entries = Vec::with_capacity(rows.len());
        for row in rows {
            let photos = load_photos(&mut tx, TimeEntryId(row.id)).await?;
            entries.push(row.into_time_entry(photos));
        }

        Ok(entries)
    }

    async fn close(
        &mut self,
        id: TimeEntryId,
        ended_at: DateTime<Utc>,
        after_the_fact: bool,
    ) -> Result<TimeEntry, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TimeEntryRow,
            r#"
            UPDATE time_entries
            SET ended_at = $2, closed_after_the_fact = $3, updated_at = now()
            WHERE id = $1 AND ended_at IS NULL
            RETURNING id, org_id, task_id, employee_id, started_at, ended_at, closed_after_the_fact, created_at, updated_at
            "#,
            id.0,
            ended_at,
            after_the_fact,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        // `ended_at IS NULL` in the WHERE makes this idempotent-safe: a second
        // close matches nothing rather than overwriting the first one's time.
        .ok_or(CoreError::NotFound)?;

        let photos = load_photos(&mut tx, id).await?;

        Ok(row.into_time_entry(photos))
    }

    async fn attach_photo(&mut self, photo: &TimeEntryPhoto) -> Result<TimeEntryPhoto, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TimeEntryPhotoRow,
            r#"
            INSERT INTO time_entry_photos (id, org_id, time_entry_id, phase, storage_key, created_at)
            VALUES ($1, $2, $3, CAST($4 AS text)::time_entry_photo_phase, $5, $6)
            RETURNING id, org_id, time_entry_id, phase::text AS "phase!", storage_key, created_at
            "#,
            photo.id.0,
            photo.organization_id.0,
            photo.time_entry_id.0,
            photo.phase.as_str(),
            photo.storage_key,
            photo.created_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.into_photo()
    }
}

/// Takes the connection rather than the guard: the callers already hold the
/// lock, and handing the guard down would need a deref level that changes with
/// the transaction type.
async fn load_photos(
    conn: &mut sqlx::PgConnection,
    time_entry_id: TimeEntryId,
) -> Result<Vec<TimeEntryPhoto>, CoreError> {
    let rows = sqlx::query_as!(
        TimeEntryPhotoRow,
        r#"
        SELECT id, org_id, time_entry_id, phase::text AS "phase!", storage_key, created_at
        FROM time_entry_photos
        WHERE time_entry_id = $1
        ORDER BY created_at
        "#,
        time_entry_id.0,
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(map_sqlx_error)?;

    rows.into_iter()
        .map(TimeEntryPhotoRow::into_photo)
        .collect()
}

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
    /// Upsert rather than insert: an employee correcting the time they went
    /// home is restating the same fact, not logging a second day.
    async fn upsert(&mut self, day_log: &DayLog) -> Result<DayLog, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DayLogRow,
            r#"
            INSERT INTO day_logs (id, org_id, employee_id, work_date, ended_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (employee_id, work_date)
            DO UPDATE SET ended_at = EXCLUDED.ended_at
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

    async fn find_for_employee_on(
        &mut self,
        employee_id: EmployeeId,
        work_date: NaiveDate,
    ) -> Result<Option<DayLog>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DayLogRow,
            r#"
            SELECT id, org_id, employee_id, work_date, ended_at, created_at
            FROM day_logs
            WHERE employee_id = $1 AND work_date = $2
            "#,
            employee_id.0,
            work_date,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(DayLogRow::into_day_log))
    }

    async fn list_by_organization_on(
        &mut self,
        organization_id: OrganizationId,
        work_date: NaiveDate,
    ) -> Result<Vec<DayLog>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            DayLogRow,
            r#"
            SELECT id, org_id, employee_id, work_date, ended_at, created_at
            FROM day_logs
            WHERE org_id = $1 AND work_date = $2
            ORDER BY ended_at
            "#,
            organization_id.0,
            work_date,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(DayLogRow::into_day_log).collect())
    }
}

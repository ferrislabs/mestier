use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    OrganizationId, TaskRecurrence, TaskRecurrenceId,
    domain::task_recurrence::{RecurrenceRule, ports::TaskRecurrenceRepository, weekday_to_iso},
    infrastructure::{
        postgres::{SharedTx, error::map_sqlx_error},
        task_recurrence::postgres::model::TaskRecurrenceRow,
    },
};

#[repository(domain = TaskRecurrence, backend = Postgres)]
pub struct PgTaskRecurrenceRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgTaskRecurrenceRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

/// Decomposes a rule into the three columns that carry it — exactly one of
/// `weekly_weekdays`/`monthly_day` populated, matching `chk_task_recurrences_weekly_days`
/// and `chk_task_recurrences_monthly_day`.
fn rule_columns(rule: &RecurrenceRule) -> (&'static str, Option<Vec<i16>>, Option<i16>) {
    match rule {
        RecurrenceRule::Daily => ("DAILY", None, None),
        RecurrenceRule::Weekly { weekdays } => (
            "WEEKLY",
            Some(weekdays.iter().copied().map(weekday_to_iso).collect()),
            None,
        ),
        RecurrenceRule::Monthly { day_of_month } => ("MONTHLY", None, Some(*day_of_month as i16)),
    }
}

impl<'tx> TaskRecurrenceRepository for PgTaskRecurrenceRepository<'tx> {
    async fn insert(&mut self, recurrence: &TaskRecurrence) -> Result<TaskRecurrence, CoreError> {
        let mut tx = self.tx.lock().await;
        let (frequency, weekly_weekdays, monthly_day) = rule_columns(&recurrence.rule);
        let assignee_member_ids: Vec<uuid::Uuid> = recurrence
            .assignee_member_ids
            .iter()
            .map(|id| id.0)
            .collect();

        let row = sqlx::query_as!(
            TaskRecurrenceRow,
            r#"
            INSERT INTO task_recurrences (
                id, org_id, frequency, weekly_weekdays, monthly_day, starts_on, ends_on,
                horizon_filled_to, timezone, start_time, duration_minutes, all_day, title,
                description, blocks_availability, customer_id, customer_context_id, project_id,
                assignee_member_ids, deleted_at, created_at, updated_at
            )
            VALUES (
                $1, $2, CAST($3 AS text)::task_recurrence_frequency, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22
            )
            RETURNING id, org_id, frequency::text AS "frequency!", weekly_weekdays, monthly_day,
                starts_on, ends_on, horizon_filled_to, timezone, start_time, duration_minutes,
                all_day, title, description, blocks_availability, customer_id,
                customer_context_id, project_id, assignee_member_ids, deleted_at, created_at,
                updated_at
            "#,
            recurrence.id.0,
            recurrence.organization_id.0,
            frequency,
            weekly_weekdays.as_deref(),
            monthly_day,
            recurrence.starts_on,
            recurrence.ends_on,
            recurrence.horizon_filled_to,
            recurrence.timezone.to_string(),
            recurrence.start_time,
            recurrence.duration_minutes,
            recurrence.all_day,
            recurrence.title,
            recurrence.description,
            recurrence.blocks_availability,
            recurrence.customer_id.map(|id| id.0),
            recurrence.customer_context_id.map(|id| id.0),
            recurrence.project_id.map(|id| id.0),
            &assignee_member_ids,
            recurrence.deleted_at,
            recurrence.created_at,
            recurrence.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.into_recurrence()
    }

    async fn find_by_id(
        &mut self,
        id: TaskRecurrenceId,
    ) -> Result<Option<TaskRecurrence>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TaskRecurrenceRow,
            r#"
            SELECT id, org_id, frequency::text AS "frequency!", weekly_weekdays, monthly_day,
                starts_on, ends_on, horizon_filled_to, timezone, start_time, duration_minutes,
                all_day, title, description, blocks_availability, customer_id,
                customer_context_id, project_id, assignee_member_ids, deleted_at, created_at,
                updated_at
            FROM task_recurrences
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(TaskRecurrenceRow::into_recurrence).transpose()
    }

    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<TaskRecurrence>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            TaskRecurrenceRow,
            r#"
            SELECT id, org_id, frequency::text AS "frequency!", weekly_weekdays, monthly_day,
                starts_on, ends_on, horizon_filled_to, timezone, start_time, duration_minutes,
                all_day, title, description, blocks_availability, customer_id,
                customer_context_id, project_id, assignee_member_ids, deleted_at, created_at,
                updated_at
            FROM task_recurrences
            WHERE org_id = $1 AND deleted_at IS NULL
            ORDER BY created_at ASC, id ASC
            "#,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter()
            .map(TaskRecurrenceRow::into_recurrence)
            .collect()
    }

    async fn update(&mut self, recurrence: &TaskRecurrence) -> Result<TaskRecurrence, CoreError> {
        let mut tx = self.tx.lock().await;
        let (frequency, weekly_weekdays, monthly_day) = rule_columns(&recurrence.rule);
        let assignee_member_ids: Vec<uuid::Uuid> = recurrence
            .assignee_member_ids
            .iter()
            .map(|id| id.0)
            .collect();

        let row = sqlx::query_as!(
            TaskRecurrenceRow,
            r#"
            UPDATE task_recurrences
            SET frequency = CAST($2 AS text)::task_recurrence_frequency,
                weekly_weekdays = $3,
                monthly_day = $4,
                ends_on = $5,
                start_time = $6,
                duration_minutes = $7,
                all_day = $8,
                title = $9,
                description = $10,
                blocks_availability = $11,
                project_id = $12,
                assignee_member_ids = $13,
                updated_at = $14
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, org_id, frequency::text AS "frequency!", weekly_weekdays, monthly_day,
                starts_on, ends_on, horizon_filled_to, timezone, start_time, duration_minutes,
                all_day, title, description, blocks_availability, customer_id,
                customer_context_id, project_id, assignee_member_ids, deleted_at, created_at,
                updated_at
            "#,
            recurrence.id.0,
            frequency,
            weekly_weekdays.as_deref(),
            monthly_day,
            recurrence.ends_on,
            recurrence.start_time,
            recurrence.duration_minutes,
            recurrence.all_day,
            recurrence.title,
            recurrence.description,
            recurrence.blocks_availability,
            recurrence.project_id.map(|id| id.0),
            &assignee_member_ids,
            recurrence.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .ok_or(CoreError::NotFound)?;

        row.into_recurrence()
    }

    async fn advance_horizon(
        &mut self,
        id: TaskRecurrenceId,
        horizon_filled_to: NaiveDate,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let affected = sqlx::query!(
            r#"
            UPDATE task_recurrences
            SET horizon_filled_to = $2, updated_at = now()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
            horizon_filled_to,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .rows_affected();

        if affected == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }

    async fn soft_delete(
        &mut self,
        id: TaskRecurrenceId,
        deleted_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let affected = sqlx::query!(
            r#"
            UPDATE task_recurrences
            SET deleted_at = $2, updated_at = $2
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
            deleted_at,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .rows_affected();

        if affected == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }
}

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{
    MemberId, OrganizationId, Task, TaskAssignment, TaskId,
    domain::task::ports::TaskRepository,
    infrastructure::{
        postgres::{SharedTx, error::map_sqlx_error},
        task::postgres::model::{TaskAssignmentRow, TaskRow},
    },
};

#[repository(domain = Task, backend = Postgres)]
pub struct PgTaskRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgTaskRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> TaskRepository for PgTaskRepository<'tx> {
    async fn insert(&mut self, task: &Task) -> Result<Task, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TaskRow,
            r#"
            INSERT INTO tasks (id, org_id, parent_task_id, title, description, starts_at, ends_at, all_day, status, blocks_availability, customer_id, customer_context_id, quote_id, deleted_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, CAST($9 AS text)::task_status, $10, $11, $12, $13, $14, $15, $16)
            RETURNING id, org_id, parent_task_id, title, description, starts_at, ends_at, all_day, status::text AS "status!", blocks_availability, customer_id, customer_context_id, quote_id, deleted_at, created_at, updated_at
            "#,
            task.id.0,
            task.organization_id.0,
            task.parent_task_id.map(|id| id.0),
            task.title,
            task.description,
            task.starts_at,
            task.ends_at,
            task.all_day,
            task.status.as_str(),
            task.blocks_availability,
            task.customer_id.map(|id| id.0),
            task.customer_context_id.map(|id| id.0),
            task.quote_id.map(|id| id.0),
            task.deleted_at,
            task.created_at,
            task.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        replace_assignments(&mut tx, task.id, &task.assignments).await?;
        row.into_task(task.assignments.clone())
    }

    async fn find_by_id(&mut self, id: TaskId) -> Result<Option<Task>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TaskRow,
            r#"
            SELECT id, org_id, parent_task_id, title, description, starts_at, ends_at, all_day, status::text AS "status!", blocks_availability, customer_id, customer_context_id, quote_id, deleted_at, created_at, updated_at
            FROM tasks
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        match row {
            Some(row) => {
                let assignments = fetch_assignments(&mut tx, id).await?;
                Ok(Some(row.into_task(assignments)?))
            }
            None => Ok(None),
        }
    }

    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        parent_task_id: Option<TaskId>,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Task>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let parent_task_id = parent_task_id.map(|id| id.0);
        let rows = sqlx::query_as!(
            TaskRow,
            r#"
            SELECT id, org_id, parent_task_id, title, description, starts_at, ends_at, all_day, status::text AS "status!", blocks_availability, customer_id, customer_context_id, quote_id, deleted_at, created_at, updated_at
            FROM tasks
            WHERE org_id = $1 AND deleted_at IS NULL
              AND (
                ($2::uuid IS NULL AND parent_task_id IS NULL)
                OR parent_task_id = $2
              )
            ORDER BY starts_at ASC NULLS LAST, id ASC
            LIMIT $3 OFFSET $4
            "#,
            organization_id.0,
            parent_task_id,
            limit as i64,
            offset as i64,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let total: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM tasks
            WHERE org_id = $1 AND deleted_at IS NULL
              AND (
                ($2::uuid IS NULL AND parent_task_id IS NULL)
                OR parent_task_id = $2
              )
            "#,
            organization_id.0,
            parent_task_id,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
        let mut assignments_by_task = fetch_assignments_for_tasks(&mut tx, &ids).await?;

        let mut tasks = Vec::with_capacity(rows.len());
        for row in rows {
            let assignments = assignments_by_task.remove(&row.id).unwrap_or_default();
            tasks.push(row.into_task(assignments)?);
        }

        Ok((tasks, total as u64))
    }

    async fn list_for_assignee_on(
        &mut self,
        organization_id: OrganizationId,
        member_id: MemberId,
        day_starts_at: DateTime<Utc>,
        day_ends_at: DateTime<Utc>,
    ) -> Result<Vec<Task>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            TaskRow,
            r#"
            SELECT t.id, t.org_id, t.parent_task_id, t.title, t.description, t.starts_at, t.ends_at, t.all_day, t.status::text AS "status!", t.blocks_availability, t.customer_id, t.customer_context_id, t.quote_id, t.deleted_at, t.created_at, t.updated_at
            FROM tasks t
            JOIN task_assignments a ON a.task_id = t.id
            WHERE t.org_id = $1
              AND a.member_id = $2
              AND t.deleted_at IS NULL
              AND (
                t.starts_at IS NULL
                OR (t.starts_at < $4 AND COALESCE(t.ends_at, t.starts_at) >= $3)
              )
            ORDER BY t.starts_at ASC NULLS LAST, t.id ASC
            "#,
            organization_id.0,
            member_id.0,
            day_starts_at,
            day_ends_at,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
        let mut assignments_by_task = fetch_assignments_for_tasks(&mut tx, &ids).await?;

        rows.into_iter()
            .map(|row| {
                let assignments = assignments_by_task.remove(&row.id).unwrap_or_default();
                row.into_task(assignments)
            })
            .collect()
    }

    async fn count_children(
        &mut self,
        task_ids: &[TaskId],
    ) -> Result<HashMap<TaskId, i64>, CoreError> {
        if task_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut tx = self.tx.lock().await;
        let ids: Vec<Uuid> = task_ids.iter().map(|id| id.0).collect();
        let rows = sqlx::query!(
            r#"
            SELECT parent_task_id AS "parent_task_id!", COUNT(*) AS "count!"
            FROM tasks
            WHERE parent_task_id = ANY($1) AND deleted_at IS NULL
            GROUP BY parent_task_id
            "#,
            &ids,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows
            .into_iter()
            .map(|row| (TaskId(row.parent_task_id), row.count))
            .collect())
    }

    async fn update(&mut self, task: &Task) -> Result<Task, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TaskRow,
            r#"
            UPDATE tasks
            SET parent_task_id = $2,
                title = $3,
                description = $4,
                starts_at = $5,
                ends_at = $6,
                all_day = $7,
                status = CAST($8 AS text)::task_status,
                blocks_availability = $9,
                updated_at = $10
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, org_id, parent_task_id, title, description, starts_at, ends_at, all_day, status::text AS "status!", blocks_availability, customer_id, customer_context_id, quote_id, deleted_at, created_at, updated_at
            "#,
            task.id.0,
            task.parent_task_id.map(|id| id.0),
            task.title,
            task.description,
            task.starts_at,
            task.ends_at,
            task.all_day,
            task.status.as_str(),
            task.blocks_availability,
            task.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let row = row.ok_or(CoreError::NotFound)?;

        replace_assignments(&mut tx, task.id, &task.assignments).await?;
        row.into_task(task.assignments.clone())
    }

    async fn soft_delete(
        &mut self,
        id: TaskId,
        deleted_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE tasks
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

async fn fetch_assignments(
    conn: &mut PgConnection,
    task_id: TaskId,
) -> Result<Vec<TaskAssignment>, CoreError> {
    let rows = sqlx::query_as!(
        TaskAssignmentRow,
        r#"
        SELECT id, org_id, task_id, member_id, created_at
        FROM task_assignments
        WHERE task_id = $1
        ORDER BY created_at ASC, id ASC
        "#,
        task_id.0,
    )
    .fetch_all(conn)
    .await
    .map_err(map_sqlx_error)?;

    Ok(rows.into_iter().map(Into::into).collect())
}

/// Every assignment for each id in `task_ids`, in one grouped query — never
/// one per task. Mirrors `count_children`'s own batching (`WHERE ... = ANY($1)`
/// plus in-memory grouping); `list_by_organization` used to call
/// `fetch_assignments` once per row inside a loop, a pre-existing N+1 found
/// while proving `GET /tasks`'s total query count for the task-labels
/// workstream (#142) — fixed here rather than left for a separate PR, since
/// T5's list view (#145) is about to hit this same endpoint hard. A task
/// with no assignments is absent from the map rather than mapped to an
/// empty `Vec`, same contract as `count_children`.
async fn fetch_assignments_for_tasks(
    conn: &mut PgConnection,
    task_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<TaskAssignment>>, CoreError> {
    if task_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query_as!(
        TaskAssignmentRow,
        r#"
        SELECT id, org_id, task_id, member_id, created_at
        FROM task_assignments
        WHERE task_id = ANY($1)
        ORDER BY task_id ASC, created_at ASC, id ASC
        "#,
        task_ids,
    )
    .fetch_all(conn)
    .await
    .map_err(map_sqlx_error)?;

    let mut grouped: HashMap<Uuid, Vec<TaskAssignment>> = HashMap::new();
    for row in rows {
        grouped.entry(row.task_id).or_default().push(row.into());
    }

    Ok(grouped)
}

/// Replaces the complete assignment set for a task: physical delete of
/// whatever is there, then insert of the new list. Assignments carry no
/// history worth keeping once removed (unlike tasks themselves, which are
/// soft-deleted), and the `PATCH` contract treats `assignees` as the full
/// list rather than a delta, so a diff would add complexity without adding
/// correctness.
async fn replace_assignments(
    conn: &mut PgConnection,
    task_id: TaskId,
    assignments: &[TaskAssignment],
) -> Result<(), CoreError> {
    sqlx::query!(
        r#"DELETE FROM task_assignments WHERE task_id = $1"#,
        task_id.0,
    )
    .execute(&mut *conn)
    .await
    .map_err(map_sqlx_error)?;

    for assignment in assignments {
        sqlx::query!(
            r#"
            INSERT INTO task_assignments (id, org_id, task_id, member_id, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            assignment.id.0,
            assignment.organization_id.0,
            assignment.task_id.0,
            assignment.member_id.0,
            assignment.created_at,
        )
        .execute(&mut *conn)
        .await
        .map_err(map_sqlx_error)?;
    }

    Ok(())
}

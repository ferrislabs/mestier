use common::CoreError;
use mestier_macros::repository;

use crate::{
    Intervention, InterventionId, InterventionStatus, MaintenanceLineId, MemberId, OrganizationId,
    TaskId,
    domain::maintenance::ports::{InterventionRepository, TaskMaintenanceContext},
    infrastructure::{
        maintenance_intervention::postgres::model::InterventionRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = MaintenanceIntervention, backend = Postgres)]
pub struct PgInterventionRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgInterventionRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> InterventionRepository for PgInterventionRepository<'tx> {
    async fn insert(&mut self, intervention: &Intervention) -> Result<Intervention, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            InterventionRow,
            r#"
            INSERT INTO maintenance_interventions (
                id, org_id, line_id, done_on, status, task_id, proposed_by,
                resolved_by, resolved_at, note, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, org_id, line_id, done_on, status, task_id, proposed_by,
                      resolved_by, resolved_at, note, created_at
            "#,
            intervention.id.0,
            intervention.organization_id.0,
            intervention.line_id.0,
            intervention.done_on,
            intervention.status.as_str(),
            intervention.task_id.map(|id| id.0),
            intervention.proposed_by.0,
            intervention.resolved_by.map(|id| id.0),
            intervention.resolved_at,
            intervention.note,
            intervention.created_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.try_into()
    }

    async fn find_by_id(&mut self, id: InterventionId) -> Result<Option<Intervention>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            InterventionRow,
            r#"
            SELECT id, org_id, line_id, done_on, status, task_id, proposed_by,
                   resolved_by, resolved_at, note, created_at
            FROM maintenance_interventions
            WHERE id = $1
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(TryInto::try_into).transpose()
    }

    async fn update_resolution(
        &mut self,
        intervention: &Intervention,
    ) -> Result<Intervention, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            InterventionRow,
            r#"
            UPDATE maintenance_interventions
            SET status = $3,
                resolved_by = $4,
                resolved_at = $5,
                note = $6
            WHERE id = $1 AND org_id = $2
            RETURNING id, org_id, line_id, done_on, status, task_id, proposed_by,
                      resolved_by, resolved_at, note, created_at
            "#,
            intervention.id.0,
            intervention.organization_id.0,
            intervention.status.as_str(),
            intervention.resolved_by.map(|id| id.0),
            intervention.resolved_at,
            intervention.note,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(TryInto::try_into)
            .transpose()?
            .ok_or(CoreError::NotFound)
    }

    async fn list_for_lines(
        &mut self,
        line_ids: &[MaintenanceLineId],
    ) -> Result<Vec<Intervention>, CoreError> {
        let ids: Vec<_> = line_ids.iter().map(|id| id.0).collect();
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            InterventionRow,
            r#"
            SELECT id, org_id, line_id, done_on, status, task_id, proposed_by,
                   resolved_by, resolved_at, note, created_at
            FROM maintenance_interventions
            WHERE line_id = ANY($1)
            ORDER BY done_on ASC, created_at ASC, id ASC
            "#,
            &ids,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn list_by_status(
        &mut self,
        organization_id: OrganizationId,
        status: InterventionStatus,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Intervention>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            InterventionRow,
            r#"
            SELECT id, org_id, line_id, done_on, status, task_id, proposed_by,
                   resolved_by, resolved_at, note, created_at
            FROM maintenance_interventions
            WHERE org_id = $1 AND status = $2
            ORDER BY created_at ASC, id ASC
            LIMIT $3 OFFSET $4
            "#,
            organization_id.0,
            status.as_str(),
            limit as i64,
            offset as i64,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let total: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM maintenance_interventions
            WHERE org_id = $1 AND status = $2
            "#,
            organization_id.0,
            status.as_str(),
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

    async fn find_task_context(
        &mut self,
        task_id: TaskId,
    ) -> Result<Option<TaskMaintenanceContext>, CoreError> {
        let mut tx = self.tx.lock().await;
        let task = sqlx::query!(
            r#"
            SELECT org_id, maintenance_line_id
            FROM tasks
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            task_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let Some(task) = task else {
            return Ok(None);
        };

        let assignees = sqlx::query_scalar!(
            r#"
            SELECT member_id
            FROM task_assignments
            WHERE task_id = $1
            "#,
            task_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(Some(TaskMaintenanceContext {
            organization_id: OrganizationId(task.org_id),
            line_id: task.maintenance_line_id.map(MaintenanceLineId),
            assignee_member_ids: assignees.into_iter().map(MemberId).collect(),
        }))
    }
}

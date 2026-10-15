use std::collections::{HashMap, HashSet};

use common::CoreError;
use mestier_macros::repository;
use uuid::Uuid;

use crate::{
    MaintenanceContract, MaintenanceContractId, MaintenanceLine, MaintenanceLineId, OrganizationId,
    domain::maintenance::ports::MaintenanceContractRepository,
    infrastructure::{
        maintenance_contract::postgres::model::{ContractRow, LineRow},
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = MaintenanceContract, backend = Postgres)]
pub struct PgMaintenanceContractRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgMaintenanceContractRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }

    async fn lines_of(
        &self,
        contract_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, Vec<MaintenanceLine>>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            LineRow,
            r#"
            SELECT id, contract_id, label, cadence, minutes_per_intervention
            FROM maintenance_lines
            WHERE contract_id = ANY($1)
            ORDER BY contract_id, position ASC, created_at ASC
            "#,
            contract_ids,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let mut grouped: HashMap<Uuid, Vec<MaintenanceLine>> = HashMap::new();
        for row in rows {
            let contract_id = row.contract_id;
            grouped
                .entry(contract_id)
                .or_default()
                .push(MaintenanceLine::try_from(row)?);
        }
        Ok(grouped)
    }

    async fn assemble(
        &self,
        rows: Vec<ContractRow>,
    ) -> Result<Vec<MaintenanceContract>, CoreError> {
        let ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
        let mut lines = self.lines_of(&ids).await?;
        rows.into_iter()
            .map(|row| {
                let lines = lines.remove(&row.id).unwrap_or_default();
                row.into_contract(lines)
            })
            .collect()
    }

    async fn upsert_line(
        &self,
        contract: &MaintenanceContract,
        line: &MaintenanceLine,
        position: i32,
    ) -> Result<(), CoreError> {
        let cadence = serde_json::to_value(&line.cadence)
            .map_err(|error| CoreError::Internal(format!("cadence cannot be stored: {error}")))?;
        let minutes = line
            .minutes_per_intervention
            .map(i32::try_from)
            .transpose()
            .map_err(|_| {
                CoreError::Conflict("minutes per intervention are too large".to_owned())
            })?;

        let mut tx = self.tx.lock().await;
        sqlx::query!(
            r#"
            INSERT INTO maintenance_lines (id, org_id, contract_id, label, cadence, minutes_per_intervention, position)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE
            SET label = EXCLUDED.label,
                cadence = EXCLUDED.cadence,
                minutes_per_intervention = EXCLUDED.minutes_per_intervention,
                position = EXCLUDED.position,
                updated_at = now()
            WHERE maintenance_lines.contract_id = EXCLUDED.contract_id
              AND maintenance_lines.org_id = EXCLUDED.org_id
            RETURNING id
            "#,
            line.id.0,
            contract.organization_id.0,
            contract.id.0,
            line.label,
            cadence,
            minutes,
            position,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}

impl<'tx> MaintenanceContractRepository for PgMaintenanceContractRepository<'tx> {
    async fn insert(
        &mut self,
        contract: &MaintenanceContract,
    ) -> Result<MaintenanceContract, CoreError> {
        let row = {
            let mut tx = self.tx.lock().await;
            sqlx::query_as!(
                ContractRow,
                r#"
                INSERT INTO maintenance_contracts (
                    id, org_id, customer_id, customer_context_id, project_id, reference,
                    season_year, first_week, last_week, manager_member_id, amount_cents,
                    quote_id, notes, status, archived_at, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
                RETURNING id, org_id, customer_id, customer_context_id, project_id, reference,
                          season_year, first_week, last_week, manager_member_id, amount_cents,
                          quote_id, notes, status, archived_at, created_at, updated_at
                "#,
                contract.id.0,
                contract.organization_id.0,
                contract.customer_id.0,
                contract.customer_context_id.0,
                contract.project_id.0,
                contract.reference,
                contract.season.year() as i16,
                i16::from(contract.season.first_week().get()),
                i16::from(contract.season.last_week().get()),
                contract.manager_member_id.map(|id| id.0),
                contract.amount_cents,
                contract.quote_id.map(|id| id.0),
                contract.notes,
                contract.status.as_str(),
                contract.archived_at,
                contract.created_at,
                contract.updated_at,
            )
            .fetch_one(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
        };

        for (position, line) in contract.lines.iter().enumerate() {
            self.upsert_line(contract, line, position as i32).await?;
        }

        let lines = self
            .lines_of(&[row.id])
            .await?
            .remove(&row.id)
            .unwrap_or_default();
        row.into_contract(lines)
    }

    async fn find_by_id(
        &mut self,
        id: MaintenanceContractId,
    ) -> Result<Option<MaintenanceContract>, CoreError> {
        let row = {
            let mut tx = self.tx.lock().await;
            sqlx::query_as!(
                ContractRow,
                r#"
                SELECT id, org_id, customer_id, customer_context_id, project_id, reference,
                       season_year, first_week, last_week, manager_member_id, amount_cents,
                       quote_id, notes, status, archived_at, created_at, updated_at
                FROM maintenance_contracts
                WHERE id = $1
                "#,
                id.0,
            )
            .fetch_optional(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
        };

        match row {
            Some(row) => Ok(self.assemble(vec![row]).await?.into_iter().next()),
            None => Ok(None),
        }
    }

    async fn find_by_line(
        &mut self,
        line_id: MaintenanceLineId,
    ) -> Result<Option<MaintenanceContract>, CoreError> {
        let row = {
            let mut tx = self.tx.lock().await;
            sqlx::query_as!(
                ContractRow,
                r#"
                SELECT c.id, c.org_id, c.customer_id, c.customer_context_id, c.project_id, c.reference,
                       c.season_year, c.first_week, c.last_week, c.manager_member_id, c.amount_cents,
                       c.quote_id, c.notes, c.status, c.archived_at, c.created_at, c.updated_at
                FROM maintenance_contracts c
                JOIN maintenance_lines l ON l.contract_id = c.id AND l.org_id = c.org_id
                WHERE l.id = $1
                "#,
                line_id.0,
            )
            .fetch_optional(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
        };

        match row {
            Some(row) => Ok(self.assemble(vec![row]).await?.into_iter().next()),
            None => Ok(None),
        }
    }

    async fn list_by_season(
        &mut self,
        organization_id: OrganizationId,
        year: i32,
        include_archived: bool,
    ) -> Result<Vec<MaintenanceContract>, CoreError> {
        let rows = {
            let mut tx = self.tx.lock().await;
            sqlx::query_as!(
                ContractRow,
                r#"
                SELECT id, org_id, customer_id, customer_context_id, project_id, reference,
                       season_year, first_week, last_week, manager_member_id, amount_cents,
                       quote_id, notes, status, archived_at, created_at, updated_at
                FROM maintenance_contracts
                WHERE org_id = $1
                  AND season_year = $2
                  AND ($3::boolean OR archived_at IS NULL)
                ORDER BY reference ASC NULLS LAST, created_at ASC
                "#,
                organization_id.0,
                year as i16,
                include_archived,
            )
            .fetch_all(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
        };

        self.assemble(rows).await
    }

    async fn update(
        &mut self,
        contract: &MaintenanceContract,
    ) -> Result<MaintenanceContract, CoreError> {
        let row = {
            let mut tx = self.tx.lock().await;
            sqlx::query_as!(
                ContractRow,
                r#"
                UPDATE maintenance_contracts
                SET customer_id = $3, customer_context_id = $4, reference = $5,
                    season_year = $6, first_week = $7, last_week = $8,
                    manager_member_id = $9, amount_cents = $10, quote_id = $11,
                    notes = $12, status = $13, archived_at = $14, updated_at = $15
                WHERE id = $1 AND org_id = $2
                RETURNING id, org_id, customer_id, customer_context_id, project_id, reference,
                          season_year, first_week, last_week, manager_member_id, amount_cents,
                          quote_id, notes, status, archived_at, created_at, updated_at
                "#,
                contract.id.0,
                contract.organization_id.0,
                contract.customer_id.0,
                contract.customer_context_id.0,
                contract.reference,
                contract.season.year() as i16,
                i16::from(contract.season.first_week().get()),
                i16::from(contract.season.last_week().get()),
                contract.manager_member_id.map(|id| id.0),
                contract.amount_cents,
                contract.quote_id.map(|id| id.0),
                contract.notes,
                contract.status.as_str(),
                contract.archived_at,
                contract.updated_at,
            )
            .fetch_optional(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
            .ok_or(CoreError::NotFound)?
        };

        let kept: HashSet<Uuid> = contract.lines.iter().map(|line| line.id.0).collect();
        let removed: Vec<Uuid> = {
            let mut tx = self.tx.lock().await;
            sqlx::query_scalar!(
                "SELECT id FROM maintenance_lines WHERE contract_id = $1",
                contract.id.0,
            )
            .fetch_all(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
            .into_iter()
            .filter(|id| !kept.contains(id))
            .collect()
        };

        if !removed.is_empty() {
            let mut tx = self.tx.lock().await;
            let has_history = sqlx::query_scalar!(
                r#"SELECT EXISTS(SELECT 1 FROM maintenance_interventions WHERE line_id = ANY($1)) AS "exists!""#,
                &removed,
            )
            .fetch_one(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
            if has_history {
                return Err(CoreError::Conflict(
                    "a line with interventions cannot be removed".to_owned(),
                ));
            }

            sqlx::query!(
                "DELETE FROM maintenance_lines WHERE contract_id = $1 AND id = ANY($2)",
                contract.id.0,
                &removed,
            )
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        }

        for (position, line) in contract.lines.iter().enumerate() {
            self.upsert_line(contract, line, position as i32).await?;
        }

        let lines = self
            .lines_of(&[row.id])
            .await?
            .remove(&row.id)
            .unwrap_or_default();
        row.into_contract(lines)
    }

    async fn lines_with_open_task(
        &mut self,
        organization_id: OrganizationId,
        line_ids: &[MaintenanceLineId],
    ) -> Result<HashSet<MaintenanceLineId>, CoreError> {
        let ids: Vec<Uuid> = line_ids.iter().map(|id| id.0).collect();
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_scalar!(
            r#"
            SELECT DISTINCT maintenance_line_id AS "line_id!"
            FROM tasks
            WHERE org_id = $1
              AND maintenance_line_id = ANY($2)
              AND deleted_at IS NULL
              AND status::text NOT IN ('DONE', 'CANCELLED')
            "#,
            organization_id.0,
            &ids,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(MaintenanceLineId).collect())
    }
}

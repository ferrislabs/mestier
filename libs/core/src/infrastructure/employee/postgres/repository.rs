use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    Employee, EmployeeId, OrganizationId, UserId,
    domain::employee::ports::EmployeeRepository,
    infrastructure::{
        employee::postgres::model::EmployeeRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = Employee, backend = Postgres)]
pub struct PgEmployeeRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgEmployeeRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> EmployeeRepository for PgEmployeeRepository<'tx> {
    async fn insert(&mut self, employee: &Employee) -> Result<Employee, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            EmployeeRow,
            r#"
            INSERT INTO employees (id, org_id, user_id, last_name, first_name, hourly_rate_cents, weekly_contract_minutes, deleted_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, org_id, user_id, last_name, first_name, hourly_rate_cents, weekly_contract_minutes, deleted_at, created_at, updated_at
            "#,
            employee.id.0,
            employee.organization_id.0,
            employee.user_id.map(|id| id.0),
            employee.last_name,
            employee.first_name,
            employee.hourly_rate_cents,
            employee.weekly_contract_minutes,
            employee.deleted_at,
            employee.created_at,
            employee.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn find_by_id(&mut self, id: EmployeeId) -> Result<Option<Employee>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            EmployeeRow,
            r#"
            SELECT id, org_id, user_id, last_name, first_name, hourly_rate_cents, weekly_contract_minutes, deleted_at, created_at, updated_at
            FROM employees
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    async fn find_by_user_id(
        &mut self,
        organization_id: OrganizationId,
        user_id: UserId,
    ) -> Result<Option<Employee>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            EmployeeRow,
            r#"
            SELECT id, org_id, user_id, last_name, first_name, hourly_rate_cents, weekly_contract_minutes, deleted_at, created_at, updated_at
            FROM employees
            WHERE org_id = $1 AND user_id = $2 AND deleted_at IS NULL
            "#,
            organization_id.0,
            user_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Employee>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            EmployeeRow,
            r#"
            SELECT id, org_id, user_id, last_name, first_name, hourly_rate_cents, weekly_contract_minutes, deleted_at, created_at, updated_at
            FROM employees
            WHERE org_id = $1 AND deleted_at IS NULL
            ORDER BY last_name ASC, first_name ASC, created_at ASC
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
            r#"SELECT COUNT(*) AS "count!" FROM employees WHERE org_id = $1 AND deleted_at IS NULL"#,
            organization_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok((rows.into_iter().map(Into::into).collect(), total as u64))
    }

    async fn list_active_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Employee>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            EmployeeRow,
            r#"
            SELECT id, org_id, user_id, last_name, first_name, hourly_rate_cents, weekly_contract_minutes, deleted_at, created_at, updated_at
            FROM employees
            WHERE org_id = $1 AND deleted_at IS NULL
            ORDER BY last_name ASC, first_name ASC, created_at ASC
            "#,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn update(&mut self, employee: &Employee) -> Result<Employee, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            EmployeeRow,
            r#"
            UPDATE employees
            SET user_id = $2, last_name = $3, first_name = $4, hourly_rate_cents = $5, weekly_contract_minutes = $6, updated_at = $7
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, org_id, user_id, last_name, first_name, hourly_rate_cents, weekly_contract_minutes, deleted_at, created_at, updated_at
            "#,
            employee.id.0,
            employee.user_id.map(|id| id.0),
            employee.last_name,
            employee.first_name,
            employee.hourly_rate_cents,
            employee.weekly_contract_minutes,
            employee.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }

    async fn soft_delete(
        &mut self,
        id: EmployeeId,
        deleted_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE employees
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

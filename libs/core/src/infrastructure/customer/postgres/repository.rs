use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    Customer, CustomerId, OrganizationId,
    domain::customer::ports::CustomerRepository,
    infrastructure::{
        customer::postgres::model::CustomerRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = Customer, backend = Postgres)]
pub struct PgCustomerRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgCustomerRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> CustomerRepository for PgCustomerRepository<'tx> {
    async fn insert(&mut self, customer: &Customer) -> Result<Customer, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CustomerRow,
            r#"
            INSERT INTO customers (id, org_id, status, pipeline_stage, name, registration_number, phone, email, deleted_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, org_id, status, pipeline_stage, name, registration_number, phone, email, deleted_at, created_at, updated_at
            "#,
            customer.id.0,
            customer.organization_id.0,
            customer.status.as_str(),
            customer.pipeline_stage.as_str(),
            customer.name,
            customer.registration_number,
            customer.phone,
            customer.email,
            customer.deleted_at,
            customer.created_at,
            customer.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.try_into()
    }

    async fn find_by_id(&mut self, id: CustomerId) -> Result<Option<Customer>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CustomerRow,
            r#"
            SELECT id, org_id, status, pipeline_stage, name, registration_number, phone, email, deleted_at, created_at, updated_at
            FROM customers
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(TryInto::try_into).transpose()
    }

    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Customer>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            CustomerRow,
            r#"
            SELECT id, org_id, status, pipeline_stage, name, registration_number, phone, email, deleted_at, created_at, updated_at
            FROM customers
            WHERE org_id = $1 AND deleted_at IS NULL
            ORDER BY name ASC, created_at ASC
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
            r#"SELECT COUNT(*) AS "count!" FROM customers WHERE org_id = $1 AND deleted_at IS NULL"#,
            organization_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let customers = rows
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, CoreError>>()?;

        Ok((customers, total as u64))
    }

    async fn update(&mut self, customer: &Customer) -> Result<Customer, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CustomerRow,
            r#"
            UPDATE customers
            SET status = $2, pipeline_stage = $3, name = $4, registration_number = $5, phone = $6, email = $7, updated_at = $8
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, org_id, status, pipeline_stage, name, registration_number, phone, email, deleted_at, created_at, updated_at
            "#,
            customer.id.0,
            customer.status.as_str(),
            customer.pipeline_stage.as_str(),
            customer.name,
            customer.registration_number,
            customer.phone,
            customer.email,
            customer.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(TryInto::try_into)
            .transpose()?
            .ok_or(CoreError::NotFound)
    }

    async fn soft_delete(
        &mut self,
        id: CustomerId,
        deleted_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE customers
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

        sqlx::query!(
            r#"
            UPDATE customer_contexts
            SET deleted_at = $2, updated_at = $2
            WHERE customer_id = $1 AND deleted_at IS NULL
            "#,
            id.0,
            deleted_at,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        sqlx::query!(
            r#"
            UPDATE customer_contacts
            SET deleted_at = $2, updated_at = $2
            WHERE customer_id = $1 AND deleted_at IS NULL
            "#,
            id.0,
            deleted_at,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}

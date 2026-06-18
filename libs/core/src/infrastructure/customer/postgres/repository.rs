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
            INSERT INTO customers (id, org_id, last_name, first_name, phone, email, deleted_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, org_id, last_name, first_name, phone, email, deleted_at, created_at, updated_at
            "#,
            customer.id.0,
            customer.organization_id.0,
            customer.last_name,
            customer.first_name,
            customer.phone,
            customer.email,
            customer.deleted_at,
            customer.created_at,
            customer.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn find_by_id(&mut self, id: CustomerId) -> Result<Option<Customer>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CustomerRow,
            r#"
            SELECT id, org_id, last_name, first_name, phone, email, deleted_at, created_at, updated_at
            FROM customers
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
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
    ) -> Result<(Vec<Customer>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            CustomerRow,
            r#"
            SELECT id, org_id, last_name, first_name, phone, email, deleted_at, created_at, updated_at
            FROM customers
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
            r#"SELECT COUNT(*) AS "count!" FROM customers WHERE org_id = $1 AND deleted_at IS NULL"#,
            organization_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok((rows.into_iter().map(Into::into).collect(), total as u64))
    }

    async fn update(&mut self, customer: &Customer) -> Result<Customer, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CustomerRow,
            r#"
            UPDATE customers
            SET last_name = $2, first_name = $3, phone = $4, email = $5, updated_at = $6
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, org_id, last_name, first_name, phone, email, deleted_at, created_at, updated_at
            "#,
            customer.id.0,
            customer.last_name,
            customer.first_name,
            customer.phone,
            customer.email,
            customer.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
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
            UPDATE properties
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

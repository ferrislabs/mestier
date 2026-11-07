use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    CustomerContext, CustomerContextId, CustomerId,
    domain::customer_context::ports::CustomerContextRepository,
    infrastructure::{
        customer_context::postgres::model::CustomerContextRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = CustomerContext, backend = Postgres)]
pub struct PgCustomerContextRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgCustomerContextRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> CustomerContextRepository for PgCustomerContextRepository<'tx> {
    async fn insert(
        &mut self,
        customer_context: &CustomerContext,
    ) -> Result<CustomerContext, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CustomerContextRow,
            r#"
            INSERT INTO customer_contexts (id, customer_id, label, kind, address_line, address_line2, postal_code, city, country, photo_key, deleted_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id, customer_id, label, kind, address_line, address_line2, postal_code, city, country, photo_key, deleted_at, created_at, updated_at
            "#,
            customer_context.id.0,
            customer_context.customer_id.0,
            customer_context.label,
            customer_context.kind.as_str(),
            customer_context.address_line,
            customer_context.address_line2,
            customer_context.postal_code,
            customer_context.city,
            customer_context.country,
            customer_context.photo_key,
            customer_context.deleted_at,
            customer_context.created_at,
            customer_context.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.try_into()
    }

    async fn find_by_id(
        &mut self,
        id: CustomerContextId,
    ) -> Result<Option<CustomerContext>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CustomerContextRow,
            r#"
            SELECT id, customer_id, label, kind, address_line, address_line2, postal_code, city, country, photo_key, deleted_at, created_at, updated_at
            FROM customer_contexts
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(TryInto::try_into).transpose()
    }

    async fn list_by_customer(
        &mut self,
        customer_id: CustomerId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<CustomerContext>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            CustomerContextRow,
            r#"
            SELECT id, customer_id, label, kind, address_line, address_line2, postal_code, city, country, photo_key, deleted_at, created_at, updated_at
            FROM customer_contexts
            WHERE customer_id = $1 AND deleted_at IS NULL
            ORDER BY label ASC, created_at ASC
            LIMIT $2 OFFSET $3
            "#,
            customer_id.0,
            limit as i64,
            offset as i64,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM customer_contexts WHERE customer_id = $1 AND deleted_at IS NULL"#,
            customer_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let contexts = rows
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, CoreError>>()?;

        Ok((contexts, total as u64))
    }

    async fn update(
        &mut self,
        customer_context: &CustomerContext,
    ) -> Result<CustomerContext, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CustomerContextRow,
            r#"
            UPDATE customer_contexts
            SET label = $2, kind = $3, address_line = $4, address_line2 = $5, postal_code = $6, city = $7, country = $8, photo_key = $9, updated_at = $10
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, customer_id, label, kind, address_line, address_line2, postal_code, city, country, photo_key, deleted_at, created_at, updated_at
            "#,
            customer_context.id.0,
            customer_context.label,
            customer_context.kind.as_str(),
            customer_context.address_line,
            customer_context.address_line2,
            customer_context.postal_code,
            customer_context.city,
            customer_context.country,
            customer_context.photo_key,
            customer_context.updated_at,
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
        id: CustomerContextId,
        deleted_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE customer_contexts
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
            UPDATE customers
            SET billing_address_id = (
                    SELECT other.id
                    FROM customer_contexts other
                    WHERE other.customer_id = customers.id
                      AND other.deleted_at IS NULL
                      AND other.kind = 'BILLING'
                    ORDER BY other.created_at ASC
                    LIMIT 1
                ),
                updated_at = $2
            WHERE billing_address_id = $1
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

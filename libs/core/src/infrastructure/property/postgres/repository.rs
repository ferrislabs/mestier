use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    CustomerId, Property, PropertyId,
    domain::property::ports::PropertyRepository,
    infrastructure::{
        postgres::{SharedTx, error::map_sqlx_error},
        property::postgres::model::PropertyRow,
    },
};

#[repository(domain = Property, backend = Postgres)]
pub struct PgPropertyRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgPropertyRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> PropertyRepository for PgPropertyRepository<'tx> {
    async fn insert(&mut self, property: &Property) -> Result<Property, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            PropertyRow,
            r#"
            INSERT INTO properties (id, customer_id, label, street, zip, city, photo_key, deleted_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, customer_id, label, street, zip, city, photo_key, deleted_at, created_at, updated_at
            "#,
            property.id.0,
            property.customer_id.0,
            property.label,
            property.street,
            property.zip,
            property.city,
            property.photo_key,
            property.deleted_at,
            property.created_at,
            property.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn find_by_id(&mut self, id: PropertyId) -> Result<Option<Property>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            PropertyRow,
            r#"
            SELECT id, customer_id, label, street, zip, city, photo_key, deleted_at, created_at, updated_at
            FROM properties
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    async fn list_by_customer(
        &mut self,
        customer_id: CustomerId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Property>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            PropertyRow,
            r#"
            SELECT id, customer_id, label, street, zip, city, photo_key, deleted_at, created_at, updated_at
            FROM properties
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
            r#"SELECT COUNT(*) AS "count!" FROM properties WHERE customer_id = $1 AND deleted_at IS NULL"#,
            customer_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok((rows.into_iter().map(Into::into).collect(), total as u64))
    }

    async fn update(&mut self, property: &Property) -> Result<Property, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            PropertyRow,
            r#"
            UPDATE properties
            SET label = $2, street = $3, zip = $4, city = $5, photo_key = $6, updated_at = $7
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, customer_id, label, street, zip, city, photo_key, deleted_at, created_at, updated_at
            "#,
            property.id.0,
            property.label,
            property.street,
            property.zip,
            property.city,
            property.photo_key,
            property.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }

    async fn soft_delete(
        &mut self,
        id: PropertyId,
        deleted_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE properties
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

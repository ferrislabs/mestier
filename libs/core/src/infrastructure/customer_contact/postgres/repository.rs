use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    CustomerContact, CustomerContactId, CustomerId,
    domain::customer_contact::ports::CustomerContactRepository,
    infrastructure::{
        customer_contact::postgres::model::CustomerContactRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = CustomerContact, backend = Postgres)]
pub struct PgCustomerContactRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgCustomerContactRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> CustomerContactRepository for PgCustomerContactRepository<'tx> {
    async fn insert(
        &mut self,
        customer_contact: &CustomerContact,
    ) -> Result<CustomerContact, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CustomerContactRow,
            r#"
            INSERT INTO customer_contacts (id, customer_id, first_name, last_name, role, phone, email, is_primary, deleted_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, customer_id, first_name, last_name, role, phone, email, is_primary, deleted_at, created_at, updated_at
            "#,
            customer_contact.id.0,
            customer_contact.customer_id.0,
            customer_contact.first_name,
            customer_contact.last_name,
            customer_contact.role,
            customer_contact.phone,
            customer_contact.email,
            customer_contact.is_primary,
            customer_contact.deleted_at,
            customer_contact.created_at,
            customer_contact.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn find_by_id(
        &mut self,
        id: CustomerContactId,
    ) -> Result<Option<CustomerContact>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CustomerContactRow,
            r#"
            SELECT id, customer_id, first_name, last_name, role, phone, email, is_primary, deleted_at, created_at, updated_at
            FROM customer_contacts
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
    ) -> Result<(Vec<CustomerContact>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            CustomerContactRow,
            r#"
            SELECT id, customer_id, first_name, last_name, role, phone, email, is_primary, deleted_at, created_at, updated_at
            FROM customer_contacts
            WHERE customer_id = $1 AND deleted_at IS NULL
            ORDER BY is_primary DESC, last_name ASC, first_name ASC, created_at ASC
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
            r#"SELECT COUNT(*) AS "count!" FROM customer_contacts WHERE customer_id = $1 AND deleted_at IS NULL"#,
            customer_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok((rows.into_iter().map(Into::into).collect(), total as u64))
    }

    async fn update(
        &mut self,
        customer_contact: &CustomerContact,
    ) -> Result<CustomerContact, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CustomerContactRow,
            r#"
            UPDATE customer_contacts
            SET first_name = $2, last_name = $3, role = $4, phone = $5, email = $6, is_primary = $7, updated_at = $8
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, customer_id, first_name, last_name, role, phone, email, is_primary, deleted_at, created_at, updated_at
            "#,
            customer_contact.id.0,
            customer_contact.first_name,
            customer_contact.last_name,
            customer_contact.role,
            customer_contact.phone,
            customer_contact.email,
            customer_contact.is_primary,
            customer_contact.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }

    async fn soft_delete(
        &mut self,
        id: CustomerContactId,
        deleted_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE customer_contacts
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

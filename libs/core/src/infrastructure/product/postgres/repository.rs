use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
	OrganizationId, Product, ProductId,
	domain::product::ports::ProductRepository,
	infrastructure::{
		postgres::{SharedTx, error::map_sqlx_error},
		product::postgres::model::ProductRow,
	},
};

#[repository(domain = Product, backend = Postgres)]
pub struct PgProductRepository<'tx> {
	tx: SharedTx<'tx>,
}

impl<'tx> PgProductRepository<'tx> {
	pub fn new(tx: &SharedTx<'tx>) -> Self {
		Self { tx: tx.clone() }
	}
}

impl<'tx> ProductRepository for PgProductRepository<'tx> {
	async fn insert(&mut self, product: &Product) -> Result<Product, CoreError> {
		let mut tx = self.tx.lock().await;
		let row = sqlx::query_as!(
			ProductRow,
			r#"
            INSERT INTO products (id, org_id, name, sku, unit, unit_price_cents, vat_rate, custom_fields, description, deleted_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, org_id, name, sku, unit, unit_price_cents, vat_rate,
                      custom_fields as "custom_fields: sqlx::types::Json<serde_json::Value>",
                      description, deleted_at, created_at, updated_at
            "#,
			product.id.0,
			product.organization_id.0,
			product.name,
			product.sku,
			product.unit.as_str(),
			product.unit_price_cents,
			product.vat_rate,
			sqlx::types::Json(&product.custom_fields) as _,
			product.description,
			product.deleted_at,
			product.created_at,
			product.updated_at,
		)
		.fetch_one(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;

		row.try_into()
	}

	async fn find_by_id(&mut self, id: ProductId) -> Result<Option<Product>, CoreError> {
		let mut tx = self.tx.lock().await;
		let row = sqlx::query_as!(
			ProductRow,
			r#"
            SELECT id, org_id, name, sku, unit, unit_price_cents, vat_rate,
                   custom_fields as "custom_fields: sqlx::types::Json<serde_json::Value>",
                   description, deleted_at, created_at, updated_at
            FROM products
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
	) -> Result<(Vec<Product>, u64), CoreError> {
		let mut tx = self.tx.lock().await;
		let rows = sqlx::query_as!(
			ProductRow,
			r#"
            SELECT id, org_id, name, sku, unit, unit_price_cents, vat_rate,
                   custom_fields as "custom_fields: sqlx::types::Json<serde_json::Value>",
                   description, deleted_at, created_at, updated_at
            FROM products
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
			r#"SELECT COUNT(*) AS "count!" FROM products WHERE org_id = $1 AND deleted_at IS NULL"#,
			organization_id.0,
		)
		.fetch_one(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;

		let items = rows
			.into_iter()
			.map(TryInto::try_into)
			.collect::<Result<Vec<_>, CoreError>>()?;

		Ok((items, total as u64))
	}

	async fn update(&mut self, product: &Product) -> Result<Product, CoreError> {
		let mut tx = self.tx.lock().await;
		let row = sqlx::query_as!(
			ProductRow,
			r#"
            UPDATE products
            SET name = $2, sku = $3, unit = $4, unit_price_cents = $5, vat_rate = $6, custom_fields = $7, description = $8, updated_at = $9
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, org_id, name, sku, unit, unit_price_cents, vat_rate,
                      custom_fields as "custom_fields: sqlx::types::Json<serde_json::Value>",
                      description, deleted_at, created_at, updated_at
            "#,
			product.id.0,
			product.name,
			product.sku,
			product.unit.as_str(),
			product.unit_price_cents,
			product.vat_rate,
			sqlx::types::Json(&product.custom_fields) as _,
			product.description,
			product.updated_at,
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
		id: ProductId,
		deleted_at: DateTime<Utc>,
	) -> Result<(), CoreError> {
		let mut tx = self.tx.lock().await;
		let result = sqlx::query!(
			r#"
            UPDATE products
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

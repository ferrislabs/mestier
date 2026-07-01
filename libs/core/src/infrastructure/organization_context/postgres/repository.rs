use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
	OrganizationContext, OrganizationContextId, OrganizationId,
	domain::organization_context::ports::OrganizationContextRepository,
	infrastructure::{
		organization_context::postgres::model::OrganizationContextRow,
		postgres::{SharedTx, error::map_sqlx_error},
	},
};

#[repository(domain = OrganizationContext, backend = Postgres)]
pub struct PgOrganizationContextRepository<'tx> {
	tx: SharedTx<'tx>,
}

impl<'tx> PgOrganizationContextRepository<'tx> {
	pub fn new(tx: &SharedTx<'tx>) -> Self {
		Self { tx: tx.clone() }
	}
}

impl<'tx> OrganizationContextRepository for PgOrganizationContextRepository<'tx> {
	async fn insert(
		&mut self,
		organization_context: &OrganizationContext,
	) -> Result<OrganizationContext, CoreError> {
		let mut tx = self.tx.lock().await;
		let row = sqlx::query_as!(
			OrganizationContextRow,
			r#"
            INSERT INTO organization_contexts (id, org_id, label, address_line, postal_code, city, country, siret, rcs, ape, vat_intracom, iban, bic, deleted_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING id, org_id, label, address_line, postal_code, city, country, siret, rcs, ape, vat_intracom, iban, bic, deleted_at, created_at, updated_at
            "#,
			organization_context.id.0,
			organization_context.org_id.0,
			organization_context.label,
			organization_context.address_line,
			organization_context.postal_code,
			organization_context.city,
			organization_context.country,
			organization_context.siret,
			organization_context.rcs,
			organization_context.ape,
			organization_context.vat_intracom,
			organization_context.iban,
			organization_context.bic,
			organization_context.deleted_at,
			organization_context.created_at,
			organization_context.updated_at,
		)
		.fetch_one(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;

		Ok(row.into())
	}

	async fn find_by_id(
		&mut self,
		id: OrganizationContextId,
	) -> Result<Option<OrganizationContext>, CoreError> {
		let mut tx = self.tx.lock().await;
		let row = sqlx::query_as!(
			OrganizationContextRow,
			r#"
            SELECT id, org_id, label, address_line, postal_code, city, country, siret, rcs, ape, vat_intracom, iban, bic, deleted_at, created_at, updated_at
            FROM organization_contexts
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
		org_id: OrganizationId,
		limit: u64,
		offset: u64,
	) -> Result<(Vec<OrganizationContext>, u64), CoreError> {
		let mut tx = self.tx.lock().await;
		let rows = sqlx::query_as!(
			OrganizationContextRow,
			r#"
            SELECT id, org_id, label, address_line, postal_code, city, country, siret, rcs, ape, vat_intracom, iban, bic, deleted_at, created_at, updated_at
            FROM organization_contexts
            WHERE org_id = $1 AND deleted_at IS NULL
            ORDER BY label ASC, created_at ASC
            LIMIT $2 OFFSET $3
            "#,
			org_id.0,
			limit as i64,
			offset as i64,
		)
		.fetch_all(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;

		let total: i64 = sqlx::query_scalar!(
			r#"SELECT COUNT(*) AS "count!" FROM organization_contexts WHERE org_id = $1 AND deleted_at IS NULL"#,
			org_id.0,
		)
		.fetch_one(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;

		Ok((rows.into_iter().map(Into::into).collect(), total as u64))
	}

	async fn update(
		&mut self,
		organization_context: &OrganizationContext,
	) -> Result<OrganizationContext, CoreError> {
		let mut tx = self.tx.lock().await;
		let row = sqlx::query_as!(
			OrganizationContextRow,
			r#"
            UPDATE organization_contexts
            SET label = $2, address_line = $3, postal_code = $4, city = $5, country = $6, siret = $7, rcs = $8, ape = $9, vat_intracom = $10, iban = $11, bic = $12, updated_at = $13
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, org_id, label, address_line, postal_code, city, country, siret, rcs, ape, vat_intracom, iban, bic, deleted_at, created_at, updated_at
            "#,
			organization_context.id.0,
			organization_context.label,
			organization_context.address_line,
			organization_context.postal_code,
			organization_context.city,
			organization_context.country,
			organization_context.siret,
			organization_context.rcs,
			organization_context.ape,
			organization_context.vat_intracom,
			organization_context.iban,
			organization_context.bic,
			organization_context.updated_at,
		)
		.fetch_optional(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;

		row.map(Into::into).ok_or(CoreError::NotFound)
	}

	async fn soft_delete(
		&mut self,
		id: OrganizationContextId,
		deleted_at: DateTime<Utc>,
	) -> Result<(), CoreError> {
		let mut tx = self.tx.lock().await;
		let result = sqlx::query!(
			r#"
            UPDATE organization_contexts
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

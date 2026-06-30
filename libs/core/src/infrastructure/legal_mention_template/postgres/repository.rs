use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
	LegalMentionTemplate, LegalMentionTemplateId, OrganizationId,
	domain::legal_mention_template::ports::LegalMentionTemplateRepository,
	infrastructure::{
		legal_mention_template::postgres::model::LegalMentionTemplateRow,
		postgres::{SharedTx, error::map_sqlx_error},
	},
};

#[repository(domain = LegalMentionTemplate, backend = Postgres)]
pub struct PgLegalMentionTemplateRepository<'tx> {
	tx: SharedTx<'tx>,
}

impl<'tx> PgLegalMentionTemplateRepository<'tx> {
	pub fn new(tx: &SharedTx<'tx>) -> Self {
		Self { tx: tx.clone() }
	}
}

impl<'tx> LegalMentionTemplateRepository for PgLegalMentionTemplateRepository<'tx> {
	async fn insert(
		&mut self,
		template: &LegalMentionTemplate,
	) -> Result<LegalMentionTemplate, CoreError> {
		let mut tx = self.tx.lock().await;
		let row = sqlx::query_as!(
			LegalMentionTemplateRow,
			r#"
            INSERT INTO legal_mention_templates (id, org_id, name, body, created_at, updated_at, deleted_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, org_id, name, body, created_at, updated_at, deleted_at
            "#,
			template.id.0,
			template.org_id.0,
			template.name,
			template.body,
			template.created_at,
			template.updated_at,
			template.deleted_at,
		)
		.fetch_one(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;

		row.try_into()
	}

	async fn find_by_id(
		&mut self,
		id: LegalMentionTemplateId,
	) -> Result<Option<LegalMentionTemplate>, CoreError> {
		let mut tx = self.tx.lock().await;
		let row = sqlx::query_as!(
			LegalMentionTemplateRow,
			r#"
            SELECT id, org_id, name, body, created_at, updated_at, deleted_at
            FROM legal_mention_templates
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
		org_id: OrganizationId,
		limit: u64,
		offset: u64,
	) -> Result<(Vec<LegalMentionTemplate>, u64), CoreError> {
		let mut tx = self.tx.lock().await;
		let rows = sqlx::query_as!(
			LegalMentionTemplateRow,
			r#"
            SELECT id, org_id, name, body, created_at, updated_at, deleted_at
            FROM legal_mention_templates
            WHERE org_id = $1 AND deleted_at IS NULL
            ORDER BY name ASC, created_at ASC
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
			r#"SELECT COUNT(*) AS "count!" FROM legal_mention_templates WHERE org_id = $1 AND deleted_at IS NULL"#,
			org_id.0,
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

	async fn update(
		&mut self,
		template: &LegalMentionTemplate,
	) -> Result<LegalMentionTemplate, CoreError> {
		let mut tx = self.tx.lock().await;
		let row = sqlx::query_as!(
			LegalMentionTemplateRow,
			r#"
            UPDATE legal_mention_templates
            SET name = $2, body = $3, updated_at = $4
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, org_id, name, body, created_at, updated_at, deleted_at
            "#,
			template.id.0,
			template.name,
			template.body,
			template.updated_at,
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
		id: LegalMentionTemplateId,
		deleted_at: DateTime<Utc>,
	) -> Result<(), CoreError> {
		let mut tx = self.tx.lock().await;
		let result = sqlx::query!(
			r#"
            UPDATE legal_mention_templates
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

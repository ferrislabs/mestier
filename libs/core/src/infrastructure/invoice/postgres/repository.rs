use chrono::{DateTime, Utc};
use common::{CoreError, generate_uuid_v7};
use mestier_macros::repository;
use serde_json::Value;
use sqlx::PgConnection;

use crate::{
	LegalMentionTemplateId, OrganizationId,
	domain::invoice::{Invoice, InvoiceId, InvoiceLine, InvoiceStatus, ports::InvoiceRepository},
	infrastructure::{
		invoice::postgres::model::{InvoiceLineRow, InvoiceRow},
		postgres::{SharedTx, error::map_sqlx_error},
	},
};

#[repository(domain = Invoice, backend = Postgres)]
pub struct PgInvoiceRepository<'tx> {
	tx: SharedTx<'tx>,
}

impl<'tx> PgInvoiceRepository<'tx> {
	pub fn new(tx: &SharedTx<'tx>) -> Self {
		Self { tx: tx.clone() }
	}
}

impl<'tx> InvoiceRepository for PgInvoiceRepository<'tx> {
	async fn next_reference(
		&mut self,
		org_id: OrganizationId,
		year: i32,
	) -> Result<String, CoreError> {
		let mut tx = self.tx.lock().await;

		// Advisory lock key: hash combining org_id with a domain-specific salt
		// ("invoice_seq") so invoice and quote counters use separate lock namespaces
		// and separate reference sequences — even for the same org in the same year.
		let lock_key = format!("invoice_seq:{}", org_id.0);
		sqlx::query!(
			r#"SELECT pg_advisory_xact_lock(hashtextextended($1::text, 0))"#,
			lock_key,
		)
		.execute(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;

		let prefix = format!("FAC-{year}-");
		let next_number: i32 = sqlx::query_scalar!(
			r#"
			SELECT COALESCE(MAX(substring(reference FROM $2)::INTEGER), 0) + 1 AS "next_number!"
			FROM invoices
			WHERE org_id = $1 AND reference LIKE $3
			"#,
			org_id.0,
			(prefix.len() + 1) as i32,
			format!("{prefix}%"),
		)
		.fetch_one(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;

		Ok(format!("{prefix}{next_number:04}"))
	}

	async fn insert(&mut self, invoice: &Invoice) -> Result<Invoice, CoreError> {
		let mut tx = self.tx.lock().await;
		let row = sqlx::query_as!(
			InvoiceRow,
			r#"
			INSERT INTO invoices (
				id, org_id, customer_id, customer_context_id, emitter_context_id, reference, title,
				status, invoice_type, source_quote_id, parent_invoice_id,
				deposit_basis, deposit_value, deposit_amount_cents,
				total_ht_cents, total_vat_cents, total_ttc_cents, total_cents,
				amount_paid_cents, pdf_key, issued_at, due_at,
				deleted_at, created_at, updated_at
			)
			VALUES (
				$1, $2, $3, $4, $5, $6, $7,
				CAST($8 AS text)::invoice_status, CAST($9 AS text)::invoice_type,
				$10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22,
				$23, $24, $25
			)
			RETURNING
				id, org_id, customer_id, customer_context_id, emitter_context_id, reference, title,
				status::text AS "status!", invoice_type::text AS "invoice_type!",
				source_quote_id, parent_invoice_id,
				deposit_basis, deposit_value, deposit_amount_cents,
				total_ht_cents, total_vat_cents, total_ttc_cents, total_cents,
				amount_paid_cents, pdf_key, issued_at, due_at,
				deleted_at, created_at, updated_at
			"#,
			invoice.id.0,
			invoice.org_id.0,
			invoice.customer_id.0,
			invoice.customer_context_id.0,
			invoice.emitter_context_id.map(|id| id.0),
			invoice.reference,
			invoice.title,
			invoice.status.as_str(),
			invoice.invoice_type.as_str(),
			invoice.source_quote_id.map(|id| id.0),
			invoice.parent_invoice_id.map(|id| id.0),
			invoice.deposit_basis,
			invoice.deposit_value,
			invoice.deposit_amount_cents,
			invoice.total_ht_cents,
			invoice.total_vat_cents,
			invoice.total_ttc_cents,
			invoice.total_cents,
			invoice.amount_paid_cents,
			invoice.pdf_key,
			invoice.issued_at,
			invoice.due_at,
			invoice.deleted_at,
			invoice.created_at,
			invoice.updated_at,
		)
		.fetch_one(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;

		insert_lines(&mut tx, &invoice.lines).await?;
		row.into_invoice(invoice.lines.clone())
	}

	async fn find_by_id(&mut self, id: InvoiceId) -> Result<Option<Invoice>, CoreError> {
		let mut tx = self.tx.lock().await;
		let row = sqlx::query_as!(
			InvoiceRow,
			r#"
			SELECT
				id, org_id, customer_id, customer_context_id, emitter_context_id, reference, title,
				status::text AS "status!", invoice_type::text AS "invoice_type!",
				source_quote_id, parent_invoice_id,
				deposit_basis, deposit_value, deposit_amount_cents,
				total_ht_cents, total_vat_cents, total_ttc_cents, total_cents,
				amount_paid_cents, pdf_key, issued_at, due_at,
				deleted_at, created_at, updated_at
			FROM invoices
			WHERE id = $1 AND deleted_at IS NULL
			"#,
			id.0,
		)
		.fetch_optional(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;

		match row {
			Some(row) => {
				let lines = fetch_lines(&mut tx, id).await?;
				Ok(Some(row.into_invoice(lines)?))
			}
			None => Ok(None),
		}
	}

	async fn list_by_organization(
		&mut self,
		org_id: OrganizationId,
		limit: u64,
		offset: u64,
	) -> Result<(Vec<Invoice>, u64), CoreError> {
		let mut tx = self.tx.lock().await;
		let rows = sqlx::query_as!(
			InvoiceRow,
			r#"
			SELECT
				id, org_id, customer_id, customer_context_id, emitter_context_id, reference, title,
				status::text AS "status!", invoice_type::text AS "invoice_type!",
				source_quote_id, parent_invoice_id,
				deposit_basis, deposit_value, deposit_amount_cents,
				total_ht_cents, total_vat_cents, total_ttc_cents, total_cents,
				amount_paid_cents, pdf_key, issued_at, due_at,
				deleted_at, created_at, updated_at
			FROM invoices
			WHERE org_id = $1 AND deleted_at IS NULL
			ORDER BY created_at DESC, id DESC
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
			r#"SELECT COUNT(*) AS "count!" FROM invoices WHERE org_id = $1 AND deleted_at IS NULL"#,
			org_id.0,
		)
		.fetch_one(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;

		let mut invoices = Vec::with_capacity(rows.len());
		for row in rows {
			let lines = fetch_lines(&mut tx, InvoiceId(row.id)).await?;
			invoices.push(row.into_invoice(lines)?);
		}

		Ok((invoices, total as u64))
	}

	async fn update(&mut self, invoice: &Invoice) -> Result<Invoice, CoreError> {
		let mut tx = self.tx.lock().await;
		let row = sqlx::query_as!(
			InvoiceRow,
			r#"
			UPDATE invoices
			SET
				title = $2,
				customer_id = $3,
				customer_context_id = $4,
				emitter_context_id = $5,
				deposit_basis = $6,
				deposit_value = $7,
				deposit_amount_cents = $8,
				total_ht_cents = $9,
				total_vat_cents = $10,
				total_ttc_cents = $11,
				total_cents = $12,
				due_at = $13,
				updated_at = $14
			WHERE id = $1 AND deleted_at IS NULL
			RETURNING
				id, org_id, customer_id, customer_context_id, emitter_context_id, reference, title,
				status::text AS "status!", invoice_type::text AS "invoice_type!",
				source_quote_id, parent_invoice_id,
				deposit_basis, deposit_value, deposit_amount_cents,
				total_ht_cents, total_vat_cents, total_ttc_cents, total_cents,
				amount_paid_cents, pdf_key, issued_at, due_at,
				deleted_at, created_at, updated_at
			"#,
			invoice.id.0,
			invoice.title,
			invoice.customer_id.0,
			invoice.customer_context_id.0,
			invoice.emitter_context_id.map(|id| id.0),
			invoice.deposit_basis,
			invoice.deposit_value,
			invoice.deposit_amount_cents,
			invoice.total_ht_cents,
			invoice.total_vat_cents,
			invoice.total_ttc_cents,
			invoice.total_cents,
			invoice.due_at,
			invoice.updated_at,
		)
		.fetch_optional(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;

		let row = row.ok_or(CoreError::NotFound)?;

		// Soft-delete existing lines, then re-insert the new set (same pattern as quote)
		sqlx::query!(
			r#"
			UPDATE invoice_lines
			SET deleted_at = $2, updated_at = $2
			WHERE invoice_id = $1 AND deleted_at IS NULL
			"#,
			invoice.id.0,
			invoice.updated_at,
		)
		.execute(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;

		insert_lines(&mut tx, &invoice.lines).await?;
		row.into_invoice(invoice.lines.clone())
	}

	async fn update_status(
		&mut self,
		id: InvoiceId,
		status: InvoiceStatus,
		reference: Option<String>,
		issued_at: Option<DateTime<Utc>>,
		updated_at: DateTime<Utc>,
	) -> Result<Invoice, CoreError> {
		let mut tx = self.tx.lock().await;
		let row = sqlx::query_as!(
			InvoiceRow,
			r#"
			UPDATE invoices
			SET
				status = CAST($2 AS text)::invoice_status,
				reference = COALESCE($3, reference),
				issued_at = COALESCE($4, issued_at),
				updated_at = $5
			WHERE id = $1 AND deleted_at IS NULL
			RETURNING
				id, org_id, customer_id, customer_context_id, emitter_context_id, reference, title,
				status::text AS "status!", invoice_type::text AS "invoice_type!",
				source_quote_id, parent_invoice_id,
				deposit_basis, deposit_value, deposit_amount_cents,
				total_ht_cents, total_vat_cents, total_ttc_cents, total_cents,
				amount_paid_cents, pdf_key, issued_at, due_at,
				deleted_at, created_at, updated_at
			"#,
			id.0,
			status.as_str(),
			reference,
			issued_at,
			updated_at,
		)
		.fetch_optional(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;

		let row = row.ok_or(CoreError::NotFound)?;
		let lines = fetch_lines(&mut tx, id).await?;
		row.into_invoice(lines)
	}

	async fn soft_delete(
		&mut self,
		id: InvoiceId,
		deleted_at: DateTime<Utc>,
	) -> Result<(), CoreError> {
		let mut tx = self.tx.lock().await;
		let result = sqlx::query!(
			r#"
			UPDATE invoices
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
			UPDATE invoice_lines
			SET deleted_at = $2, updated_at = $2
			WHERE invoice_id = $1 AND deleted_at IS NULL
			"#,
			id.0,
			deleted_at,
		)
		.execute(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;

		Ok(())
	}

	async fn replace_legal_mention_templates(
		&mut self,
		invoice: &Invoice,
		template_ids: &[LegalMentionTemplateId],
	) -> Result<(), CoreError> {
		let mut tx = self.tx.lock().await;

		sqlx::query!(
			r#"DELETE FROM invoice_legal_mentions WHERE invoice_id = $1"#,
			invoice.id.0,
		)
		.execute(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;

		insert_legal_mention_templates(&mut tx, invoice, template_ids).await
	}

	async fn find_legal_mention_template_ids(
		&mut self,
		invoice_id: InvoiceId,
	) -> Result<Vec<LegalMentionTemplateId>, CoreError> {
		let mut tx = self.tx.lock().await;
		fetch_legal_mention_template_ids(&mut tx, invoice_id).await
	}

	async fn list_deposits_for_parent(
		&mut self,
		parent_id: InvoiceId,
	) -> Result<Vec<Invoice>, CoreError> {
		let mut tx = self.tx.lock().await;
		let rows = sqlx::query_as!(
			InvoiceRow,
			r#"
			SELECT
				id, org_id, customer_id, customer_context_id, emitter_context_id, reference, title,
				status::text AS "status!", invoice_type::text AS "invoice_type!",
				source_quote_id, parent_invoice_id,
				deposit_basis, deposit_value, deposit_amount_cents,
				total_ht_cents, total_vat_cents, total_ttc_cents, total_cents,
				amount_paid_cents, pdf_key, issued_at, due_at,
				deleted_at, created_at, updated_at
			FROM invoices
			WHERE parent_invoice_id = $1
			  AND invoice_type = 'DEPOSIT'
			  AND deleted_at IS NULL
			"#,
			parent_id.0,
		)
		.fetch_all(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;

		let mut invoices = Vec::with_capacity(rows.len());
		for row in rows {
			invoices.push(row.into_invoice(vec![])?);
		}
		Ok(invoices)
	}

	async fn mark_issued(
		&mut self,
		id: InvoiceId,
		reference: String,
		issued_at: DateTime<Utc>,
		pdf_key: String,
		snapshot_json: Value,
		updated_at: DateTime<Utc>,
	) -> Result<Invoice, CoreError> {
		let mut tx = self.tx.lock().await;
		let row = sqlx::query_as!(
			InvoiceRow,
			r#"
			UPDATE invoices
			SET
				status     = 'SENT'::invoice_status,
				reference  = $2,
				issued_at  = $3,
				pdf_key    = $4,
				snapshot   = $5,
				updated_at = $6
			WHERE id = $1 AND deleted_at IS NULL
			RETURNING
				id, org_id, customer_id, customer_context_id, emitter_context_id, reference, title,
				status::text AS "status!", invoice_type::text AS "invoice_type!",
				source_quote_id, parent_invoice_id,
				deposit_basis, deposit_value, deposit_amount_cents,
				total_ht_cents, total_vat_cents, total_ttc_cents, total_cents,
				amount_paid_cents, pdf_key, issued_at, due_at,
				deleted_at, created_at, updated_at
			"#,
			id.0,
			reference,
			issued_at,
			pdf_key,
			snapshot_json,
			updated_at,
		)
		.fetch_optional(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;

		let row = row.ok_or(CoreError::NotFound)?;
		let lines = fetch_lines(&mut tx, id).await?;
		row.into_invoice(lines)
	}
}

async fn fetch_legal_mention_template_ids(
	conn: &mut PgConnection,
	invoice_id: InvoiceId,
) -> Result<Vec<LegalMentionTemplateId>, CoreError> {
	let rows = sqlx::query_scalar!(
		r#"
		SELECT template_id
		FROM invoice_legal_mentions
		WHERE invoice_id = $1
		ORDER BY created_at ASC, id ASC
		"#,
		invoice_id.0,
	)
	.fetch_all(conn)
	.await
	.map_err(map_sqlx_error)?;

	Ok(rows.into_iter().map(LegalMentionTemplateId).collect())
}

async fn insert_legal_mention_templates(
	conn: &mut PgConnection,
	invoice: &Invoice,
	template_ids: &[LegalMentionTemplateId],
) -> Result<(), CoreError> {
	for template_id in template_ids {
		sqlx::query!(
			r#"
			INSERT INTO invoice_legal_mentions (id, org_id, invoice_id, template_id)
			VALUES ($1, $2, $3, $4)
			"#,
			generate_uuid_v7(),
			invoice.org_id.0,
			invoice.id.0,
			template_id.0,
		)
		.execute(&mut *conn)
		.await
		.map_err(map_sqlx_error)?;
	}
	Ok(())
}

async fn fetch_lines(
	conn: &mut PgConnection,
	invoice_id: InvoiceId,
) -> Result<Vec<InvoiceLine>, CoreError> {
	let rows = sqlx::query_as!(
		InvoiceLineRow,
		r#"
		SELECT id, org_id, invoice_id, service_rate_id, label, quantity, unit,
		       unit_price_cents, vat_rate, notes, photo_keys, deleted_at, created_at, updated_at
		FROM invoice_lines
		WHERE invoice_id = $1 AND deleted_at IS NULL
		ORDER BY created_at ASC, id ASC
		"#,
		invoice_id.0,
	)
	.fetch_all(conn)
	.await
	.map_err(map_sqlx_error)?;

	rows.into_iter().map(TryInto::try_into).collect()
}

async fn insert_lines(conn: &mut PgConnection, lines: &[InvoiceLine]) -> Result<(), CoreError> {
	for line in lines {
		sqlx::query!(
			r#"
			INSERT INTO invoice_lines (
				id, org_id, invoice_id, service_rate_id,
				label, quantity, unit, unit_price_cents, vat_rate,
				notes, photo_keys, deleted_at, created_at, updated_at
			)
			VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
			"#,
			line.id.0,
			line.org_id.0,
			line.invoice_id.0,
			line.service_rate_id.map(|id| id.0),
			line.label,
			line.quantity,
			line.unit.as_str(),
			line.unit_price_cents,
			line.vat_rate,
			line.notes,
			&line.photo_keys,
			line.deleted_at,
			line.created_at,
			line.updated_at,
		)
		.execute(&mut *conn)
		.await
		.map_err(map_sqlx_error)?;
	}
	Ok(())
}

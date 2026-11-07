use chrono::{DateTime, NaiveDate, Utc};
use common::{CoreError, UserId};
use mestier_macros::repository;
use sqlx::PgConnection;

use crate::{
    CustomerOutstandingBalance, DraftInvoice, GeneratedInvoiceDocument, Invoice, InvoiceCollection,
    InvoiceEvent, InvoiceId, InvoiceLine, InvoiceNumberSeries, InvoicePayment, InvoicePaymentId,
    InvoiceStatus, LegalIdentity, OrganizationId, ProjectId, QuoteId,
    domain::{
        invoice::ports::InvoiceRepository,
        organization::{billing_settings::NumberPadding, numbering::format_number},
    },
    infrastructure::{
        invoice::postgres::model::{
            CustomerOutstandingBalanceRow, InvoiceCollectionRow, InvoiceEventRow, InvoiceLineRow,
            InvoicePaymentRow, InvoiceRow, delivery_address_columns, legal_identity_to_json,
            vat_breakdown_to_json,
        },
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
    async fn allocate_number(
        &mut self,
        organization_id: OrganizationId,
        series: InvoiceNumberSeries,
        prefix: &str,
        year: i32,
        padding: NumberPadding,
    ) -> Result<String, CoreError> {
        let mut tx = self.tx.lock().await;

        let sequence: i32 = match series {
            InvoiceNumberSeries::Invoice => {
                sqlx::query_scalar!(
                    r#"
                    INSERT INTO invoice_number_counters (organization_id, year, next_number)
                    VALUES ($1, $2, 1)
                    ON CONFLICT (organization_id, year)
                    DO UPDATE SET
                        next_number = invoice_number_counters.next_number + 1,
                        updated_at = now()
                    RETURNING next_number
                    "#,
                    organization_id.0,
                    year,
                )
                .fetch_one(&mut ***tx)
                .await
            }
            InvoiceNumberSeries::CreditNote => {
                sqlx::query_scalar!(
                    r#"
                    INSERT INTO credit_note_number_counters (organization_id, year, last_number)
                    VALUES ($1, $2, 1)
                    ON CONFLICT (organization_id, year)
                    DO UPDATE SET
                        last_number = credit_note_number_counters.last_number + 1,
                        updated_at = now()
                    RETURNING last_number
                    "#,
                    organization_id.0,
                    year,
                )
                .fetch_one(&mut ***tx)
                .await
            }
        }
        .map_err(map_sqlx_error)?;

        Ok(format_number(prefix, year, sequence, padding))
    }

    async fn last_assigned_number(
        &mut self,
        organization_id: OrganizationId,
        series: InvoiceNumberSeries,
        year: i32,
    ) -> Result<i32, CoreError> {
        let mut tx = self.tx.lock().await;

        let last = match series {
            InvoiceNumberSeries::Invoice => {
                sqlx::query_scalar!(
                    r#"
                    SELECT next_number
                    FROM invoice_number_counters
                    WHERE organization_id = $1 AND year = $2
                    "#,
                    organization_id.0,
                    year,
                )
                .fetch_optional(&mut ***tx)
                .await
            }
            InvoiceNumberSeries::CreditNote => {
                sqlx::query_scalar!(
                    r#"
                    SELECT last_number
                    FROM credit_note_number_counters
                    WHERE organization_id = $1 AND year = $2
                    "#,
                    organization_id.0,
                    year,
                )
                .fetch_optional(&mut ***tx)
                .await
            }
        }
        .map_err(map_sqlx_error)?;

        Ok(last.unwrap_or(0))
    }

    async fn raise_last_assigned_number(
        &mut self,
        organization_id: OrganizationId,
        series: InvoiceNumberSeries,
        year: i32,
        last_assigned: i32,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;

        match series {
            InvoiceNumberSeries::Invoice => {
                sqlx::query!(
                    r#"
                    INSERT INTO invoice_number_counters (organization_id, year, next_number)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (organization_id, year)
                    DO UPDATE SET
                        next_number = GREATEST(
                            invoice_number_counters.next_number, EXCLUDED.next_number
                        ),
                        updated_at = now()
                    "#,
                    organization_id.0,
                    year,
                    last_assigned,
                )
                .execute(&mut ***tx)
                .await
            }
            InvoiceNumberSeries::CreditNote => {
                sqlx::query!(
                    r#"
                    INSERT INTO credit_note_number_counters (organization_id, year, last_number)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (organization_id, year)
                    DO UPDATE SET
                        last_number = GREATEST(
                            credit_note_number_counters.last_number, EXCLUDED.last_number
                        ),
                        updated_at = now()
                    "#,
                    organization_id.0,
                    year,
                    last_assigned,
                )
                .execute(&mut ***tx)
                .await
            }
        }
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn insert_draft(&mut self, draft: &DraftInvoice) -> Result<Invoice, CoreError> {
        let invoice = draft.invoice();
        let mut tx = self.tx.lock().await;
        let vat_breakdown = vat_breakdown_to_json(&invoice.vat_breakdown);
        let issuer_identity = invoice.issuer_identity.as_ref().map(legal_identity_to_json);
        let operation_nature = invoice.operation_nature.map(|nature| nature.as_str());
        let (delivery_line1, delivery_line2, delivery_postal_code, delivery_city, delivery_country) =
            delivery_address_columns(&invoice.delivery_address);
        let row = sqlx::query_as!(
            InvoiceRow,
            r#"
            INSERT INTO invoices (
                id, org_id, number, kind, project_id, customer_id, customer_context_id,
                status, issued_at, due_at, notes, operation_nature,
                delivery_address_line1, delivery_address_line2, delivery_address_postal_code,
                delivery_address_city, delivery_address_country,
                net_cents, vat_breakdown, gross_cents,
                issuer_identity, source_invoice_id,
                title, payment_terms_days, legal_notice, internal_note, sent_at, quote_id,
                issued_on, reason, full_cancellation,
                deleted_at, created_at, updated_at
            )
            VALUES (
                $1, $2, $3, CAST($4 AS text)::invoice_kind, $5, $6, $7,
                CAST($8 AS text)::invoice_status, $9, $10, $11,
                CAST($12 AS text)::invoice_operation_nature,
                $13, $14, $15, $16, $17,
                $18, $19, $20,
                $21, $22,
                $23, $24, $25, $26, $27, $28,
                $29, $30, $31,
                $32, $33, $34
            )
            RETURNING
                id, org_id, number, kind::text AS "kind!", project_id, customer_id,
                customer_context_id, status::text AS "status!", issued_at, due_at, notes,
                operation_nature::text,
                delivery_address_line1, delivery_address_line2, delivery_address_postal_code,
                delivery_address_city, delivery_address_country,
                net_cents, vat_breakdown, gross_cents, issuer_identity,
                document_format, document_file_key, document_mime_type, document_generated_at,
                source_invoice_id,
                title, payment_terms_days, legal_notice, internal_note, sent_at, quote_id,
                issued_on, reason, full_cancellation,
                deleted_at, created_at, updated_at
            "#,
            invoice.id.0,
            invoice.organization_id.0,
            invoice.number,
            invoice.kind.as_str(),
            invoice.project_id.map(|id| id.0),
            invoice.customer_id.0,
            invoice.customer_context_id.0,
            invoice.status.as_str(),
            invoice.issued_at,
            invoice.due_at,
            invoice.notes,
            operation_nature,
            delivery_line1,
            delivery_line2,
            delivery_postal_code,
            delivery_city,
            delivery_country,
            invoice.net_cents,
            vat_breakdown,
            invoice.gross_cents,
            issuer_identity,
            invoice.source_invoice_id.map(|id| id.0),
            invoice.title,
            invoice.payment_terms_days,
            invoice.legal_notice,
            invoice.internal_note,
            invoice.sent_at,
            invoice.quote_id.map(|id| id.0),
            invoice.issued_on,
            invoice.reason,
            invoice.full_cancellation,
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
                id, org_id, number, kind::text AS "kind!", project_id, customer_id,
                customer_context_id, status::text AS "status!", issued_at, due_at, notes,
                operation_nature::text,
                delivery_address_line1, delivery_address_line2, delivery_address_postal_code,
                delivery_address_city, delivery_address_country,
                net_cents, vat_breakdown, gross_cents, issuer_identity,
                document_format, document_file_key, document_mime_type, document_generated_at,
                source_invoice_id,
                title, payment_terms_days, legal_notice, internal_note, sent_at, quote_id,
                issued_on, reason, full_cancellation,
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
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Invoice>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            InvoiceRow,
            r#"
            SELECT
                id, org_id, number, kind::text AS "kind!", project_id, customer_id,
                customer_context_id, status::text AS "status!", issued_at, due_at, notes,
                operation_nature::text,
                delivery_address_line1, delivery_address_line2, delivery_address_postal_code,
                delivery_address_city, delivery_address_country,
                net_cents, vat_breakdown, gross_cents, issuer_identity,
                document_format, document_file_key, document_mime_type, document_generated_at,
                source_invoice_id,
                title, payment_terms_days, legal_notice, internal_note, sent_at, quote_id,
                issued_on, reason, full_cancellation,
                deleted_at, created_at, updated_at
            FROM invoices
            WHERE org_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC, id DESC
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
            r#"SELECT COUNT(*) AS "count!" FROM invoices WHERE org_id = $1 AND deleted_at IS NULL"#,
            organization_id.0,
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

    async fn list_by_project(&mut self, project_id: ProjectId) -> Result<Vec<Invoice>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            InvoiceRow,
            r#"
            SELECT
                id, org_id, number, kind::text AS "kind!", project_id, customer_id,
                customer_context_id, status::text AS "status!", issued_at, due_at, notes,
                operation_nature::text,
                delivery_address_line1, delivery_address_line2, delivery_address_postal_code,
                delivery_address_city, delivery_address_country,
                net_cents, vat_breakdown, gross_cents, issuer_identity,
                document_format, document_file_key, document_mime_type, document_generated_at,
                source_invoice_id,
                title, payment_terms_days, legal_notice, internal_note, sent_at, quote_id,
                issued_on, reason, full_cancellation,
                deleted_at, created_at, updated_at
            FROM invoices
            WHERE project_id = $1 AND deleted_at IS NULL
            ORDER BY created_at ASC, id ASC
            "#,
            project_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let mut invoices = Vec::with_capacity(rows.len());
        for row in rows {
            let lines = fetch_lines(&mut tx, InvoiceId(row.id)).await?;
            invoices.push(row.into_invoice(lines)?);
        }

        Ok(invoices)
    }

    async fn list_by_source_invoice(
        &mut self,
        source_invoice_id: InvoiceId,
    ) -> Result<Vec<Invoice>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            InvoiceRow,
            r#"
            SELECT
                id, org_id, number, kind::text AS "kind!", project_id, customer_id,
                customer_context_id, status::text AS "status!", issued_at, due_at, notes,
                operation_nature::text,
                delivery_address_line1, delivery_address_line2, delivery_address_postal_code,
                delivery_address_city, delivery_address_country,
                net_cents, vat_breakdown, gross_cents, issuer_identity,
                document_format, document_file_key, document_mime_type, document_generated_at,
                source_invoice_id,
                title, payment_terms_days, legal_notice, internal_note, sent_at, quote_id,
                issued_on, reason, full_cancellation,
                deleted_at, created_at, updated_at
            FROM invoices
            WHERE source_invoice_id = $1 AND deleted_at IS NULL
            ORDER BY created_at ASC, id ASC
            "#,
            source_invoice_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let mut invoices = Vec::with_capacity(rows.len());
        for row in rows {
            let lines = fetch_lines(&mut tx, InvoiceId(row.id)).await?;
            invoices.push(row.into_invoice(lines)?);
        }

        Ok(invoices)
    }

    async fn update_draft(&mut self, draft: &DraftInvoice) -> Result<Invoice, CoreError> {
        let invoice = draft.invoice();
        let mut tx = self.tx.lock().await;
        let vat_breakdown = vat_breakdown_to_json(&invoice.vat_breakdown);
        let operation_nature = invoice.operation_nature.map(|nature| nature.as_str());
        let (delivery_line1, delivery_line2, delivery_postal_code, delivery_city, delivery_country) =
            delivery_address_columns(&invoice.delivery_address);
        let row = sqlx::query_as!(
            InvoiceRow,
            r#"
            UPDATE invoices
            SET project_id = $2,
                customer_id = $3,
                customer_context_id = $4,
                notes = $5,
                operation_nature = CAST($6 AS text)::invoice_operation_nature,
                delivery_address_line1 = $7,
                delivery_address_line2 = $8,
                delivery_address_postal_code = $9,
                delivery_address_city = $10,
                delivery_address_country = $11,
                net_cents = $12,
                vat_breakdown = $13,
                gross_cents = $14,
                title = $15,
                payment_terms_days = $16,
                legal_notice = $17,
                updated_at = $18
            WHERE id = $1 AND deleted_at IS NULL AND status = 'DRAFT'
            RETURNING
                id, org_id, number, kind::text AS "kind!", project_id, customer_id,
                customer_context_id, status::text AS "status!", issued_at, due_at, notes,
                operation_nature::text,
                delivery_address_line1, delivery_address_line2, delivery_address_postal_code,
                delivery_address_city, delivery_address_country,
                net_cents, vat_breakdown, gross_cents, issuer_identity,
                document_format, document_file_key, document_mime_type, document_generated_at,
                source_invoice_id,
                title, payment_terms_days, legal_notice, internal_note, sent_at, quote_id,
                issued_on, reason, full_cancellation,
                deleted_at, created_at, updated_at
            "#,
            invoice.id.0,
            invoice.project_id.map(|id| id.0),
            invoice.customer_id.0,
            invoice.customer_context_id.0,
            invoice.notes,
            operation_nature,
            delivery_line1,
            delivery_line2,
            delivery_postal_code,
            delivery_city,
            delivery_country,
            invoice.net_cents,
            vat_breakdown,
            invoice.gross_cents,
            invoice.title,
            invoice.payment_terms_days,
            invoice.legal_notice,
            invoice.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let row = row.ok_or(CoreError::NotFound)?;

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
        updated_at: DateTime<Utc>,
    ) -> Result<Invoice, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            InvoiceRow,
            r#"
            UPDATE invoices
            SET status = CAST($2 AS text)::invoice_status,
                updated_at = $3
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING
                id, org_id, number, kind::text AS "kind!", project_id, customer_id,
                customer_context_id, status::text AS "status!", issued_at, due_at, notes,
                operation_nature::text,
                delivery_address_line1, delivery_address_line2, delivery_address_postal_code,
                delivery_address_city, delivery_address_country,
                net_cents, vat_breakdown, gross_cents, issuer_identity,
                document_format, document_file_key, document_mime_type, document_generated_at,
                source_invoice_id,
                title, payment_terms_days, legal_notice, internal_note, sent_at, quote_id,
                issued_on, reason, full_cancellation,
                deleted_at, created_at, updated_at
            "#,
            id.0,
            status.as_str(),
            updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let row = row.ok_or(CoreError::NotFound)?;
        let lines = fetch_lines(&mut tx, id).await?;
        row.into_invoice(lines)
    }

    #[allow(clippy::too_many_arguments)]
    async fn issue(
        &mut self,
        id: InvoiceId,
        number: String,
        issued_at: DateTime<Utc>,
        issued_on: NaiveDate,
        due_at: Option<DateTime<Utc>>,
        issuer_identity: &LegalIdentity,
        updated_at: DateTime<Utc>,
    ) -> Result<Invoice, CoreError> {
        let mut tx = self.tx.lock().await;
        let issuer_identity_json = legal_identity_to_json(issuer_identity);
        let row = sqlx::query_as!(
            InvoiceRow,
            r#"
            UPDATE invoices
            SET number = $2,
                status = 'ISSUED'::invoice_status,
                issued_at = $3,
                issued_on = $4,
                due_at = $5,
                issuer_identity = $6,
                updated_at = $7
            WHERE id = $1 AND deleted_at IS NULL AND status = 'DRAFT'
            RETURNING
                id, org_id, number, kind::text AS "kind!", project_id, customer_id,
                customer_context_id, status::text AS "status!", issued_at, due_at, notes,
                operation_nature::text,
                delivery_address_line1, delivery_address_line2, delivery_address_postal_code,
                delivery_address_city, delivery_address_country,
                net_cents, vat_breakdown, gross_cents, issuer_identity,
                document_format, document_file_key, document_mime_type, document_generated_at,
                source_invoice_id,
                title, payment_terms_days, legal_notice, internal_note, sent_at, quote_id,
                issued_on, reason, full_cancellation,
                deleted_at, created_at, updated_at
            "#,
            id.0,
            number,
            issued_at,
            issued_on,
            due_at,
            issuer_identity_json,
            updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let row = row.ok_or(CoreError::NotFound)?;
        let lines = fetch_lines(&mut tx, id).await?;
        row.into_invoice(lines)
    }

    async fn last_issued_on(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Option<NaiveDate>, CoreError> {
        let mut tx = self.tx.lock().await;

        sqlx::query_scalar!(
            r#"
            SELECT MAX(issued_on)
            FROM invoices
            WHERE org_id = $1 AND deleted_at IS NULL AND kind <> 'CREDIT_NOTE'
            "#,
            organization_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)
    }

    async fn last_credit_note_on(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Option<NaiveDate>, CoreError> {
        let mut tx = self.tx.lock().await;

        sqlx::query_scalar!(
            r#"
            SELECT MAX(issued_on)
            FROM invoices
            WHERE org_id = $1 AND deleted_at IS NULL AND kind = 'CREDIT_NOTE'
            "#,
            organization_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)
    }

    async fn list_by_quote(&mut self, quote_id: QuoteId) -> Result<Vec<Invoice>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            InvoiceRow,
            r#"
            SELECT
                id, org_id, number, kind::text AS "kind!", project_id, customer_id,
                customer_context_id, status::text AS "status!", issued_at, due_at, notes,
                operation_nature::text,
                delivery_address_line1, delivery_address_line2, delivery_address_postal_code,
                delivery_address_city, delivery_address_country,
                net_cents, vat_breakdown, gross_cents, issuer_identity,
                document_format, document_file_key, document_mime_type, document_generated_at,
                source_invoice_id,
                title, payment_terms_days, legal_notice, internal_note, sent_at, quote_id,
                issued_on, reason, full_cancellation,
                deleted_at, created_at, updated_at
            FROM invoices
            WHERE quote_id = $1 AND deleted_at IS NULL
            ORDER BY created_at ASC, id ASC
            "#,
            quote_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let mut invoices = Vec::with_capacity(rows.len());
        for row in rows {
            let lines = fetch_lines(&mut tx, InvoiceId(row.id)).await?;
            invoices.push(row.into_invoice(lines)?);
        }

        Ok(invoices)
    }

    async fn collections(
        &mut self,
        invoice_ids: &[InvoiceId],
    ) -> Result<Vec<InvoiceCollection>, CoreError> {
        let mut tx = self.tx.lock().await;
        let ids: Vec<uuid::Uuid> = invoice_ids.iter().map(|id| id.0).collect();
        let rows = sqlx::query_as!(
            InvoiceCollectionRow,
            r#"
            SELECT
                i.id AS "invoice_id!",
                COALESCE((
                    SELECT SUM(p.amount_cents)
                    FROM invoice_payments p
                    WHERE p.invoice_id = i.id AND p.deleted_at IS NULL
                ), 0)::BIGINT AS "paid_cents!",
                COALESCE((
                    SELECT SUM(c.gross_cents)
                    FROM invoices c
                    WHERE c.source_invoice_id = i.id AND c.deleted_at IS NULL
                        AND c.status NOT IN ('DRAFT', 'CANCELLED')
                ), 0)::BIGINT AS "credited_cents!"
            FROM invoices i
            WHERE i.id = ANY($1)
            "#,
            &ids,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn list_events(&mut self, invoice_id: InvoiceId) -> Result<Vec<InvoiceEvent>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            InvoiceEventRow,
            r#"
            SELECT id, kind, detail, at, actor
            FROM invoice_events
            WHERE invoice_id = $1
            ORDER BY at ASC, id ASC
            "#,
            invoice_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn insert_event(
        &mut self,
        organization_id: OrganizationId,
        invoice_id: InvoiceId,
        event: &InvoiceEvent,
    ) -> Result<InvoiceEvent, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            InvoiceEventRow,
            r#"
            INSERT INTO invoice_events (id, org_id, invoice_id, kind, detail, at, actor)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, kind, detail, at, actor
            "#,
            event.id.0,
            organization_id.0,
            invoice_id.0,
            event.kind.as_str(),
            event.detail,
            event.at,
            event.actor.map(|actor| actor.0),
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.try_into()
    }

    async fn mark_sent(&mut self, id: InvoiceId, sent_at: DateTime<Utc>) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE invoices
            SET sent_at = $2, updated_at = $2
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
            sent_at,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }

    async fn set_internal_note(
        &mut self,
        id: InvoiceId,
        note: &str,
        updated_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE invoices
            SET internal_note = NULLIF(btrim($2), ''), updated_at = $3
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
            note,
            updated_at,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }

    async fn record_generated_document(
        &mut self,
        id: InvoiceId,
        document: &GeneratedInvoiceDocument,
        updated_at: DateTime<Utc>,
    ) -> Result<Invoice, CoreError> {
        let mut tx = self.tx.lock().await;
        // `AND document_file_key IS NULL` is the actual enforcement of
        // "set at most once" — see `GeneratedInvoiceDocument`'s own doc
        // comment. `trg_invoices_forbid_document_reassignment` is the
        // second, independent line of defence (any writer, not only this
        // one), so a row already carrying a document is simply not among
        // the rows this `UPDATE` matches, rather than reached and rejected.
        let row = sqlx::query_as!(
            InvoiceRow,
            r#"
            UPDATE invoices
            SET document_format = $2,
                document_file_key = $3,
                document_mime_type = $4,
                document_generated_at = $5,
                updated_at = $6
            WHERE id = $1 AND deleted_at IS NULL AND document_file_key IS NULL
            RETURNING
                id, org_id, number, kind::text AS "kind!", project_id, customer_id,
                customer_context_id, status::text AS "status!", issued_at, due_at, notes,
                operation_nature::text,
                delivery_address_line1, delivery_address_line2, delivery_address_postal_code,
                delivery_address_city, delivery_address_country,
                net_cents, vat_breakdown, gross_cents, issuer_identity,
                document_format, document_file_key, document_mime_type, document_generated_at,
                source_invoice_id,
                title, payment_terms_days, legal_notice, internal_note, sent_at, quote_id,
                issued_on, reason, full_cancellation,
                deleted_at, created_at, updated_at
            "#,
            id.0,
            document.format,
            document.file_key,
            document.mime_type,
            document.generated_at,
            updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let row = row.ok_or_else(|| {
            CoreError::Conflict(format!(
                "invoice {id} was not found, or already has a generated document"
            ))
        })?;
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

    async fn insert_payment(
        &mut self,
        payment: &InvoicePayment,
    ) -> Result<InvoicePayment, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            InvoicePaymentRow,
            r#"
            INSERT INTO invoice_payments (
                id, org_id, invoice_id, amount_cents, paid_on, method, reference, note,
                recorded_by, deleted_by, deleted_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING
                id, org_id, invoice_id, amount_cents, paid_on, method, reference, note,
                recorded_by, deleted_by, deleted_at, created_at, updated_at
            "#,
            payment.id.0,
            payment.organization_id.0,
            payment.invoice_id.0,
            payment.amount_cents,
            payment.paid_on,
            payment.method.as_str(),
            payment.reference,
            payment.note,
            payment.recorded_by.0,
            payment.deleted_by.map(|id| id.0),
            payment.deleted_at,
            payment.created_at,
            payment.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.try_into()
    }

    async fn find_payment_by_id(
        &mut self,
        id: InvoicePaymentId,
    ) -> Result<Option<InvoicePayment>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            InvoicePaymentRow,
            r#"
            SELECT
                id, org_id, invoice_id, amount_cents, paid_on, method, reference, note,
                recorded_by, deleted_by, deleted_at, created_at, updated_at
            FROM invoice_payments
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(TryInto::try_into).transpose()
    }

    async fn list_payments(
        &mut self,
        invoice_id: InvoiceId,
    ) -> Result<Vec<InvoicePayment>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            InvoicePaymentRow,
            r#"
            SELECT
                id, org_id, invoice_id, amount_cents, paid_on, method, reference, note,
                recorded_by, deleted_by, deleted_at, created_at, updated_at
            FROM invoice_payments
            WHERE invoice_id = $1 AND deleted_at IS NULL
            ORDER BY paid_on ASC, created_at ASC
            "#,
            invoice_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn soft_delete_payment(
        &mut self,
        id: InvoicePaymentId,
        deleted_at: DateTime<Utc>,
        deleted_by: UserId,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE invoice_payments
            SET deleted_at = $2, deleted_by = $3, updated_at = $2
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
            deleted_at,
            deleted_by.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }

    /// A SQL aggregate, deliberately: CLAUDE.md is explicit that money math
    /// lives in the backend, and a dunning list is exactly the kind of read
    /// that must never be assembled client-side from several endpoints.
    ///
    /// Same one-cent tolerance as `derive_invoice_status` (`> 1`, not
    /// `> 0`) — this query and that pure function compute the same
    /// underlying fact (is there a balance left) through two different
    /// paths for performance reasons. If the tolerance ever changes, it
    /// has to change in both places, or this list and a decorated
    /// `Invoice.status` disagree about which invoices still owe something.
    async fn list_outstanding_by_customer(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<CustomerOutstandingBalance>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            CustomerOutstandingBalanceRow,
            r#"
            SELECT
                i.customer_id AS "customer_id!",
                SUM(i.gross_cents - COALESCE(p.paid, 0) - COALESCE(c.credited, 0))::BIGINT AS "outstanding_cents!",
                MIN(i.due_at) AS oldest_due_at
            FROM invoices i
            LEFT JOIN (
                SELECT invoice_id, SUM(amount_cents) AS paid
                FROM invoice_payments
                WHERE deleted_at IS NULL
                GROUP BY invoice_id
            ) p ON p.invoice_id = i.id
            LEFT JOIN (
                SELECT source_invoice_id, SUM(gross_cents) AS credited
                FROM invoices
                WHERE deleted_at IS NULL AND status NOT IN ('DRAFT', 'CANCELLED')
                    AND source_invoice_id IS NOT NULL
                GROUP BY source_invoice_id
            ) c ON c.source_invoice_id = i.id
            -- `i.kind != 'CREDIT_NOTE'`: a credit note is itself a row in
            -- `invoices` with `status = 'ISSUED'` (see #318's header
            -- comment — it is not a fifth table), so without this filter
            -- it would be picked up twice: once correctly subtracted from
            -- its source through the `c` join above, and once counted
            -- again as its own positive outstanding balance because
            -- nothing else in this query distinguishes "a document a
            -- customer owes on" from "a document that corrects one".
            WHERE i.org_id = $1 AND i.deleted_at IS NULL AND i.status = 'ISSUED'
                AND i.kind != 'CREDIT_NOTE'
            GROUP BY i.customer_id
            HAVING SUM(i.gross_cents - COALESCE(p.paid, 0) - COALESCE(c.credited, 0)) > 1
            ORDER BY oldest_due_at ASC NULLS LAST
            "#,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Same one-cent tolerance, and the same duplication-for-performance
    /// reason, as `list_outstanding_by_customer` above — see its comment.
    async fn list_overdue(
        &mut self,
        organization_id: OrganizationId,
        as_of: DateTime<Utc>,
    ) -> Result<Vec<Invoice>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            InvoiceRow,
            r#"
            SELECT
                id, org_id, number, kind::text AS "kind!", project_id, customer_id,
                customer_context_id, status::text AS "status!", issued_at, due_at, notes,
                operation_nature::text,
                delivery_address_line1, delivery_address_line2, delivery_address_postal_code,
                delivery_address_city, delivery_address_country,
                net_cents, vat_breakdown, gross_cents, issuer_identity,
                document_format, document_file_key, document_mime_type, document_generated_at,
                source_invoice_id,
                title, payment_terms_days, legal_notice, internal_note, sent_at, quote_id,
                issued_on, reason, full_cancellation,
                deleted_at, created_at, updated_at
            FROM invoices i
            WHERE i.org_id = $1
                AND i.deleted_at IS NULL
                AND i.status = 'ISSUED'
                -- Same reason as `list_outstanding_by_customer`: a credit
                -- note is itself a row here and must never be read as a
                -- document a customer owes on. In practice a credit note
                -- always carries `due_at = NULL` (see
                -- `InvoiceService::issue_credit_note`) so the next
                -- condition already excludes it, but this says so
                -- explicitly rather than relying on that as an accident.
                AND i.kind != 'CREDIT_NOTE'
                AND i.due_at IS NOT NULL
                AND i.due_at < $2
                AND (
                    i.gross_cents
                    - COALESCE(
                        (SELECT SUM(p.amount_cents) FROM invoice_payments p
                         WHERE p.invoice_id = i.id AND p.deleted_at IS NULL),
                        0
                    )
                    - COALESCE(
                        (SELECT SUM(c.gross_cents) FROM invoices c
                         WHERE c.source_invoice_id = i.id AND c.deleted_at IS NULL
                             AND c.status NOT IN ('DRAFT', 'CANCELLED')),
                        0
                    )
                ) > 1
            ORDER BY i.due_at ASC, i.id ASC
            "#,
            organization_id.0,
            as_of,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let mut invoices = Vec::with_capacity(rows.len());
        for row in rows {
            let lines = fetch_lines(&mut tx, InvoiceId(row.id)).await?;
            invoices.push(row.into_invoice(lines)?);
        }

        Ok(invoices)
    }
}

async fn fetch_lines(
    conn: &mut PgConnection,
    invoice_id: InvoiceId,
) -> Result<Vec<InvoiceLine>, CoreError> {
    let rows = sqlx::query_as!(
        InvoiceLineRow,
        r#"
        SELECT id, org_id, invoice_id, label, quantity, unit_price_cents, vat_rate_basis_points,
               discount_percent, product_id, service_rate_id, source_line_id,
               position, deleted_at, created_at, updated_at
        FROM invoice_lines
        WHERE invoice_id = $1 AND deleted_at IS NULL
        ORDER BY position ASC, id ASC
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
                id, org_id, invoice_id, label, quantity, unit_price_cents,
                vat_rate_basis_points, discount_percent, product_id, service_rate_id,
                source_line_id, position, deleted_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
            line.id.0,
            line.organization_id.0,
            line.invoice_id.0,
            line.label,
            line.quantity,
            line.unit_price_cents,
            line.vat_rate_basis_points,
            line.discount_percent.get(),
            line.product_id.map(|id| id.0),
            line.service_rate_id.map(|id| id.0),
            line.source_line_id.map(|id| id.0),
            line.position,
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

#[cfg(test)]
mod tests {
    use common::generate_uuid_v7;
    use sqlx::PgPool;

    use super::*;
    use crate::application::test_support::{dev_pool, purge};
    use crate::infrastructure::postgres::with_tx;

    async fn seed_organization(pool: &PgPool, label: &str) -> OrganizationId {
        let owner_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            owner_id,
            format!("owner-{owner_id}@example.com"),
            format!("owner-{owner_id}"),
            "Owner User",
            format!("sub-owner-{owner_id}"),
        )
        .execute(pool)
        .await
        .unwrap();

        let org_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organizations (id, name, slug, owner_id)
               VALUES ($1, $2, $3, $4)"#,
            org_id,
            format!("{label} Org"),
            format!("{label}-{org_id}"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();

        OrganizationId(org_id)
    }

    async fn cleanup(pool: &PgPool, org_id: OrganizationId, owner_id: uuid::Uuid) {
        purge(
            pool,
            "DELETE FROM credit_note_number_counters WHERE organization_id = $1",
            org_id.0,
        )
        .await;
        purge(
            pool,
            "DELETE FROM invoice_number_counters WHERE organization_id = $1",
            org_id.0,
        )
        .await;
        purge(pool, "DELETE FROM organizations WHERE id = $1", org_id.0).await;
        purge(pool, "DELETE FROM users WHERE id = $1", owner_id).await;
    }

    /// Same guarantee as `quote::postgres::repository`'s equivalent test,
    /// against the invoice counter this time: gapless and sequential per
    /// year within one transaction, using the pattern #313 established.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn allocate_number_is_gapless_and_sequential_within_one_transaction() {
        let pool = dev_pool().await;
        let org_id = seed_organization(&pool, "invoice-sequential").await;

        let first = with_tx(&pool, async |tx| {
            let mut repo = PgInvoiceRepository::new(&tx);
            repo.allocate_number(
                org_id,
                InvoiceNumberSeries::Invoice,
                "FAC",
                2026,
                NumberPadding::DEFAULT,
            )
            .await
        })
        .await
        .unwrap();
        let second = with_tx(&pool, async |tx| {
            let mut repo = PgInvoiceRepository::new(&tx);
            repo.allocate_number(
                org_id,
                InvoiceNumberSeries::Invoice,
                "FAC",
                2026,
                NumberPadding::DEFAULT,
            )
            .await
        })
        .await
        .unwrap();
        let different_year = with_tx(&pool, async |tx| {
            let mut repo = PgInvoiceRepository::new(&tx);
            repo.allocate_number(
                org_id,
                InvoiceNumberSeries::Invoice,
                "FAC",
                2027,
                NumberPadding::DEFAULT,
            )
            .await
        })
        .await
        .unwrap();

        assert_eq!(first, "FAC-2026-0001");
        assert_eq!(second, "FAC-2026-0002");
        assert_eq!(different_year, "FAC-2027-0001");

        let owner_id =
            sqlx::query_scalar!("SELECT owner_id FROM organizations WHERE id = $1", org_id.0)
                .fetch_one(&pool)
                .await
                .unwrap();
        cleanup(&pool, org_id, owner_id).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn spec_crm_1_4_credit_notes_have_their_own_series_that_restarts_each_year() {
        let pool = dev_pool().await;
        let org_id = seed_organization(&pool, "credit-note-series").await;
        let padding = NumberPadding::try_from(5).unwrap();

        let invoice = with_tx(&pool, async |tx| {
            let mut repo = PgInvoiceRepository::new(&tx);
            repo.allocate_number(org_id, InvoiceNumberSeries::Invoice, "FAC", 2026, padding)
                .await
        })
        .await
        .unwrap();
        let first = with_tx(&pool, async |tx| {
            let mut repo = PgInvoiceRepository::new(&tx);
            repo.allocate_number(org_id, InvoiceNumberSeries::CreditNote, "AV", 2026, padding)
                .await
        })
        .await
        .unwrap();
        let second = with_tx(&pool, async |tx| {
            let mut repo = PgInvoiceRepository::new(&tx);
            repo.allocate_number(org_id, InvoiceNumberSeries::CreditNote, "AV", 2026, padding)
                .await
        })
        .await
        .unwrap();
        let next_year = with_tx(&pool, async |tx| {
            let mut repo = PgInvoiceRepository::new(&tx);
            repo.allocate_number(org_id, InvoiceNumberSeries::CreditNote, "AV", 2027, padding)
                .await
        })
        .await
        .unwrap();

        assert_eq!(invoice, "FAC-2026-00001");
        assert_eq!(first, "AV-2026-00001");
        assert_eq!(second, "AV-2026-00002");
        assert_eq!(next_year, "AV-2027-00001");

        let owner_id =
            sqlx::query_scalar!("SELECT owner_id FROM organizations WHERE id = $1", org_id.0)
                .fetch_one(&pool)
                .await
                .unwrap();
        cleanup(&pool, org_id, owner_id).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_series_counter_only_moves_forward() {
        let pool = dev_pool().await;
        let org_id = seed_organization(&pool, "counter-forward").await;

        let (last_before, last_after_raise, last_after_lower, next) = with_tx(&pool, async |tx| {
            let mut repo = PgInvoiceRepository::new(&tx);
            let series = InvoiceNumberSeries::CreditNote;
            let before = repo.last_assigned_number(org_id, series, 2026).await?;
            repo.raise_last_assigned_number(org_id, series, 2026, 119)
                .await?;
            let raised = repo.last_assigned_number(org_id, series, 2026).await?;
            repo.raise_last_assigned_number(org_id, series, 2026, 50)
                .await?;
            let lowered = repo.last_assigned_number(org_id, series, 2026).await?;
            let next = repo
                .allocate_number(org_id, series, "AV", 2026, NumberPadding::DEFAULT)
                .await?;
            Ok((before, raised, lowered, next))
        })
        .await
        .unwrap();

        assert_eq!(last_before, 0);
        assert_eq!(last_after_raise, 119);
        assert_eq!(last_after_lower, 119);
        assert_eq!(next, "AV-2026-0120");

        let owner_id =
            sqlx::query_scalar!("SELECT owner_id FROM organizations WHERE id = $1", org_id.0)
                .fetch_one(&pool)
                .await
                .unwrap();
        cleanup(&pool, org_id, owner_id).await;
    }

    /// Invoice numbering is stricter than quote numbering — a gap is a real
    /// problem, not just an inconvenience — so this asserts under real
    /// contention, not just in a comment, exactly like the quote test it
    /// mirrors.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn concurrent_allocations_for_the_same_organization_and_year_never_collide() {
        let pool = dev_pool().await;
        let org_id = seed_organization(&pool, "invoice-contention").await;

        let pool_a = pool.clone();
        let pool_b = pool.clone();

        let (a, b) = tokio::join!(
            with_tx(&pool_a, async |tx| {
                let mut repo = PgInvoiceRepository::new(&tx);
                repo.allocate_number(
                    org_id,
                    InvoiceNumberSeries::Invoice,
                    "FAC",
                    2026,
                    NumberPadding::DEFAULT,
                )
                .await
            }),
            with_tx(&pool_b, async |tx| {
                let mut repo = PgInvoiceRepository::new(&tx);
                repo.allocate_number(
                    org_id,
                    InvoiceNumberSeries::Invoice,
                    "FAC",
                    2026,
                    NumberPadding::DEFAULT,
                )
                .await
            }),
        );

        let mut numbers = [a.unwrap(), b.unwrap()];
        numbers.sort();

        assert_eq!(
            numbers,
            ["FAC-2026-0001".to_owned(), "FAC-2026-0002".to_owned()],
        );

        let owner_id =
            sqlx::query_scalar!("SELECT owner_id FROM organizations WHERE id = $1", org_id.0)
                .fetch_one(&pool)
                .await
                .unwrap();
        cleanup(&pool, org_id, owner_id).await;
    }

    // ---- #320: payments, and the two SQL aggregates over them ----

    use crate::{
        CustomerContextId, CustomerId, InvoiceKind, InvoiceLineId, OrganizationAddress, VatStatus,
    };
    use chrono::SubsecRound;
    use rust_decimal::Decimal;

    async fn seed_customer(pool: &PgPool, org_id: OrganizationId, label: &str) -> CustomerId {
        let customer_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO customers (id, org_id, name) VALUES ($1, $2, $3)"#,
            customer_id,
            org_id.0,
            format!("{label} Customer"),
        )
        .execute(pool)
        .await
        .unwrap();

        CustomerId(customer_id)
    }

    async fn seed_customer_context(pool: &PgPool, customer_id: CustomerId) -> CustomerContextId {
        let context_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO customer_contexts (id, customer_id, label) VALUES ($1, $2, $3)"#,
            context_id,
            customer_id.0,
            "Chantier principal",
        )
        .execute(pool)
        .await
        .unwrap();

        CustomerContextId(context_id)
    }

    fn stub_legal_identity() -> LegalIdentity {
        LegalIdentity {
            legal_name: "Acme SARL".to_owned(),
            legal_form: "SARL".to_owned(),
            registration_number: "123 456 789 00012".to_owned(),
            vat_status: VatStatus::NotSubject {
                basis: "art 293B du CGI".to_owned(),
            },
            share_capital_cents: None,
            address: OrganizationAddress {
                line1: "1 rue des Artisans".to_owned(),
                line2: None,
                postal_code: "75001".to_owned(),
                city: "Paris".to_owned(),
                country: "FR".to_owned(),
            },
            contact_email: None,
            contact_phone: None,
            insurance_mention: "RC Pro n°123456 - MAAF Assurances".to_owned(),
            billing: None,
        }
    }

    /// Builds and issues an invoice (or, when `source_invoice_id` is set, a
    /// credit note) with a single line whose net and gross both equal
    /// `amount_cents` — organizations in this fixture carry no VAT status,
    /// same simplification `build_single_line_draft` documents.
    #[allow(clippy::too_many_arguments)]
    async fn seed_issued_invoice(
        pool: &PgPool,
        org_id: OrganizationId,
        customer_id: CustomerId,
        customer_context_id: CustomerContextId,
        kind: InvoiceKind,
        source_invoice_id: Option<InvoiceId>,
        due_at: Option<DateTime<Utc>>,
        amount_cents: i32,
        number: &str,
    ) -> InvoiceId {
        let now = Utc::now();
        let invoice_id = InvoiceId(generate_uuid_v7());
        let line = InvoiceLine {
            id: InvoiceLineId(generate_uuid_v7()),
            organization_id: org_id,
            invoice_id,
            label: "Prestation".to_owned(),
            quantity: Decimal::ONE,
            unit_price_cents: amount_cents,
            vat_rate_basis_points: None,
            discount_percent: crate::DiscountPercent::ZERO,
            product_id: None,
            service_rate_id: None,
            source_line_id: None,
            position: 0,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        };
        let draft = DraftInvoice::try_from_invoice(Invoice {
            id: invoice_id,
            organization_id: org_id,
            number: None,
            kind,
            project_id: None,
            customer_id,
            customer_context_id,
            status: InvoiceStatus::Draft,
            issued_at: None,
            due_at,
            notes: None,
            operation_nature: None,
            delivery_address: None,
            net_cents: amount_cents,
            vat_breakdown: Vec::new(),
            gross_cents: amount_cents,
            issuer_identity: None,
            generated_document: None,
            lines: vec![line],
            source_invoice_id,
            title: String::new(),
            payment_terms_days: 30,
            legal_notice: String::new(),
            internal_note: None,
            sent_at: None,
            quote_id: None,
            issued_on: None,
            reason: (kind == InvoiceKind::CreditNote).then(|| "Test".to_owned()),
            full_cancellation: false,
            derived: crate::InvoiceDerived::default(),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        })
        .expect("just constructed with status Draft");

        with_tx(pool, async |tx| {
            let mut repo = PgInvoiceRepository::new(&tx);
            repo.insert_draft(&draft).await?;
            repo.issue(
                invoice_id,
                number.to_owned(),
                now,
                now.date_naive(),
                due_at,
                &stub_legal_identity(),
                now,
            )
            .await
        })
        .await
        .unwrap();

        invoice_id
    }

    async fn seed_payment(
        pool: &PgPool,
        org_id: OrganizationId,
        invoice_id: InvoiceId,
        amount_cents: i32,
        recorded_by: uuid::Uuid,
    ) -> InvoicePaymentId {
        let payment = InvoicePayment {
            id: InvoicePaymentId(generate_uuid_v7()),
            organization_id: org_id,
            invoice_id,
            amount_cents,
            paid_on: Utc::now().date_naive(),
            method: crate::PaymentMethod::Transfer,
            reference: None,
            note: None,
            recorded_by: UserId(recorded_by),
            deleted_at: None,
            deleted_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let inserted = with_tx(pool, async |tx| {
            let mut repo = PgInvoiceRepository::new(&tx);
            repo.insert_payment(&payment).await
        })
        .await
        .unwrap();

        inserted.id
    }

    async fn cleanup_with_customer(
        pool: &PgPool,
        org_id: OrganizationId,
        customer_id: CustomerId,
        owner_id: uuid::Uuid,
    ) {
        purge(
            pool,
            "DELETE FROM invoice_payments WHERE org_id = $1",
            org_id.0,
        )
        .await;
        // Credit notes reference their source through a self-FK; deleting
        // every row for the organization in one statement clears both
        // sides together rather than needing a specific order.
        purge(pool, "DELETE FROM invoices WHERE org_id = $1", org_id.0).await;
        purge(
            pool,
            "DELETE FROM customer_contexts WHERE customer_id = $1",
            customer_id.0,
        )
        .await;
        purge(pool, "DELETE FROM customers WHERE id = $1", customer_id.0).await;
        purge(pool, "DELETE FROM organizations WHERE id = $1", org_id.0).await;
        purge(pool, "DELETE FROM users WHERE id = $1", owner_id).await;
    }

    /// `list_overdue` is exactly the kind of query that looks right and
    /// silently isn't: it must find an issued, past-due invoice with a
    /// balance still owed, and must not find one that is either not yet
    /// due, fully paid, or already covered by a credit note.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn list_overdue_finds_only_past_due_invoices_with_a_balance_owed() {
        let pool = dev_pool().await;
        let org_id = seed_organization(&pool, "invoice-overdue").await;
        let customer_id = seed_customer(&pool, org_id, "invoice-overdue").await;
        let customer_context_id = seed_customer_context(&pool, customer_id).await;
        let owner_id =
            sqlx::query_scalar!("SELECT owner_id FROM organizations WHERE id = $1", org_id.0)
                .fetch_one(&pool)
                .await
                .unwrap();

        let as_of = Utc::now();
        // Truncated to microsecond precision: Postgres `TIMESTAMPTZ` stores
        // no finer, and the round-tripped value is compared for equality
        // below.
        let past_due = (as_of - chrono::Duration::days(10)).trunc_subsecs(6);
        let not_yet_due = (as_of + chrono::Duration::days(10)).trunc_subsecs(6);

        let owed_invoice = seed_issued_invoice(
            &pool,
            org_id,
            customer_id,
            customer_context_id,
            InvoiceKind::Standard,
            None,
            Some(past_due),
            10_000,
            "FAC-2026-9001",
        )
        .await;

        let fully_paid_invoice = seed_issued_invoice(
            &pool,
            org_id,
            customer_id,
            customer_context_id,
            InvoiceKind::Standard,
            None,
            Some(past_due),
            5_000,
            "FAC-2026-9002",
        )
        .await;
        seed_payment(&pool, org_id, fully_paid_invoice, 5_000, owner_id).await;

        let _not_yet_due_invoice = seed_issued_invoice(
            &pool,
            org_id,
            customer_id,
            customer_context_id,
            InvoiceKind::Standard,
            None,
            Some(not_yet_due),
            7_000,
            "FAC-2026-9003",
        )
        .await;

        let overdue = with_tx(&pool, async |tx| {
            let mut repo = PgInvoiceRepository::new(&tx);
            repo.list_overdue(org_id, as_of).await
        })
        .await
        .unwrap();

        let overdue_ids: Vec<InvoiceId> = overdue.iter().map(|invoice| invoice.id).collect();
        assert_eq!(
            overdue_ids,
            vec![owed_invoice],
            "only the invoice that is both past due and still owed must be listed"
        );

        cleanup_with_customer(&pool, org_id, customer_id, owner_id).await;
    }

    /// Same guarantee as `list_overdue`, on the per-customer aggregate:
    /// outstanding is gross, net of both recorded payments and credit
    /// notes, summed across every issued invoice for the customer.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn list_outstanding_by_customer_nets_payments_and_credit_notes() {
        let pool = dev_pool().await;
        let org_id = seed_organization(&pool, "invoice-outstanding").await;
        let customer_id = seed_customer(&pool, org_id, "invoice-outstanding").await;
        let customer_context_id = seed_customer_context(&pool, customer_id).await;
        let owner_id =
            sqlx::query_scalar!("SELECT owner_id FROM organizations WHERE id = $1", org_id.0)
                .fetch_one(&pool)
                .await
                .unwrap();

        let due_at = (Utc::now() - chrono::Duration::days(3)).trunc_subsecs(6);
        let invoice_id = seed_issued_invoice(
            &pool,
            org_id,
            customer_id,
            customer_context_id,
            InvoiceKind::Standard,
            None,
            Some(due_at),
            10_000,
            "FAC-2026-9101",
        )
        .await;
        seed_payment(&pool, org_id, invoice_id, 4_000, owner_id).await;
        seed_issued_invoice(
            &pool,
            org_id,
            customer_id,
            customer_context_id,
            InvoiceKind::CreditNote,
            Some(invoice_id),
            None,
            1_000,
            "FAC-2026-9102",
        )
        .await;

        let outstanding = with_tx(&pool, async |tx| {
            let mut repo = PgInvoiceRepository::new(&tx);
            repo.list_outstanding_by_customer(org_id).await
        })
        .await
        .unwrap();

        assert_eq!(outstanding.len(), 1);
        let balance = &outstanding[0];
        assert_eq!(balance.customer_id, customer_id);
        assert_eq!(
            balance.outstanding_cents, 5_000,
            "10_000 gross, minus a 4_000 payment, minus a 1_000 credit note"
        );
        assert_eq!(balance.oldest_due_at, Some(due_at));

        cleanup_with_customer(&pool, org_id, customer_id, owner_id).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn spec_crm_5_18_collections_sum_the_net_payments_and_the_credit_notes() {
        let pool = dev_pool().await;
        let org_id = seed_organization(&pool, "invoice-collections").await;
        let customer_id = seed_customer(&pool, org_id, "invoice-collections").await;
        let customer_context_id = seed_customer_context(&pool, customer_id).await;
        let owner_id =
            sqlx::query_scalar!("SELECT owner_id FROM organizations WHERE id = $1", org_id.0)
                .fetch_one(&pool)
                .await
                .unwrap();
        let seed = |kind, source, amount, number: &'static str| {
            seed_issued_invoice(
                &pool,
                org_id,
                customer_id,
                customer_context_id,
                kind,
                source,
                None,
                amount,
                number,
            )
        };
        let invoice_id = seed(InvoiceKind::Standard, None, 10_000, "FAC-2026-9201").await;
        let untouched_id = seed(InvoiceKind::Standard, None, 700, "FAC-2026-9202").await;
        seed(
            InvoiceKind::CreditNote,
            Some(invoice_id),
            1_000,
            "AV-2026-9201",
        )
        .await;
        seed_payment(&pool, org_id, invoice_id, 4_000, owner_id).await;
        seed_payment(&pool, org_id, invoice_id, -1_000, owner_id).await;

        let collections = with_tx(&pool, async |tx| {
            let mut repo = PgInvoiceRepository::new(&tx);
            repo.collections(&[invoice_id, untouched_id]).await
        })
        .await
        .unwrap();

        let of = |id: InvoiceId| {
            collections
                .iter()
                .find(|collection| collection.invoice_id == id)
                .copied()
                .expect("one row per requested invoice")
        };
        assert_eq!(of(invoice_id).paid_cents, 3_000);
        assert_eq!(of(invoice_id).credited_cents, 1_000);
        assert_eq!(of(untouched_id).paid_cents, 0);
        assert_eq!(of(untouched_id).credited_cents, 0);

        cleanup_with_customer(&pool, org_id, customer_id, owner_id).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn spec_crm_5_2_the_last_issue_dates_are_read_per_kind() {
        let pool = dev_pool().await;
        let org_id = seed_organization(&pool, "invoice-last-dates").await;
        let customer_id = seed_customer(&pool, org_id, "invoice-last-dates").await;
        let customer_context_id = seed_customer_context(&pool, customer_id).await;
        let owner_id =
            sqlx::query_scalar!("SELECT owner_id FROM organizations WHERE id = $1", org_id.0)
                .fetch_one(&pool)
                .await
                .unwrap();
        let read = |pool: &PgPool| {
            let pool = pool.clone();
            async move {
                with_tx(&pool, async |tx| {
                    let mut repo = PgInvoiceRepository::new(&tx);
                    Ok((
                        repo.last_issued_on(org_id).await?,
                        repo.last_credit_note_on(org_id).await?,
                    ))
                })
                .await
                .unwrap()
            }
        };

        assert_eq!(read(&pool).await, (None, None));

        let invoice_id = seed_issued_invoice(
            &pool,
            org_id,
            customer_id,
            customer_context_id,
            InvoiceKind::Standard,
            None,
            None,
            1_000,
            "FAC-2026-9301",
        )
        .await;
        let today = Utc::now().date_naive();
        assert_eq!(read(&pool).await, (Some(today), None));

        seed_issued_invoice(
            &pool,
            org_id,
            customer_id,
            customer_context_id,
            InvoiceKind::CreditNote,
            Some(invoice_id),
            None,
            500,
            "AV-2026-9301",
        )
        .await;
        assert_eq!(read(&pool).await, (Some(today), Some(today)));

        cleanup_with_customer(&pool, org_id, customer_id, owner_id).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn spec_crm_5_17_events_are_listed_in_order_and_sending_and_the_note_are_stored() {
        let pool = dev_pool().await;
        let org_id = seed_organization(&pool, "invoice-events").await;
        let customer_id = seed_customer(&pool, org_id, "invoice-events").await;
        let customer_context_id = seed_customer_context(&pool, customer_id).await;
        let owner_id =
            sqlx::query_scalar!("SELECT owner_id FROM organizations WHERE id = $1", org_id.0)
                .fetch_one(&pool)
                .await
                .unwrap();
        let invoice_id = seed_issued_invoice(
            &pool,
            org_id,
            customer_id,
            customer_context_id,
            InvoiceKind::Standard,
            None,
            None,
            1_000,
            "FAC-2026-9401",
        )
        .await;
        let sent_at = Utc::now().trunc_subsecs(6);
        let event = |kind, detail: &str, at: DateTime<Utc>| InvoiceEvent {
            id: crate::InvoiceEventId(generate_uuid_v7()),
            kind,
            detail: detail.to_owned(),
            at,
            actor: Some(UserId(owner_id)),
        };

        let (events, stored) = with_tx(&pool, async |tx| {
            let mut repo = PgInvoiceRepository::new(&tx);
            repo.mark_sent(invoice_id, sent_at).await?;
            repo.insert_event(
                org_id,
                invoice_id,
                &event(
                    crate::InvoiceEventKind::Reminder,
                    "Rappel envoyé",
                    sent_at + chrono::Duration::seconds(5),
                ),
            )
            .await?;
            repo.insert_event(
                org_id,
                invoice_id,
                &event(crate::InvoiceEventKind::Sent, "Envoyée au client", sent_at),
            )
            .await?;
            repo.set_internal_note(invoice_id, "  Rappeler mardi  ", sent_at)
                .await?;
            Ok((
                repo.list_events(invoice_id).await?,
                repo.find_by_id(invoice_id).await?.expect("the invoice"),
            ))
        })
        .await
        .unwrap();

        assert_eq!(
            events.iter().map(|event| event.kind).collect::<Vec<_>>(),
            vec![
                crate::InvoiceEventKind::Sent,
                crate::InvoiceEventKind::Reminder
            ]
        );
        assert_eq!(stored.sent_at, Some(sent_at));
        assert_eq!(stored.internal_note.as_deref(), Some("Rappeler mardi"));

        let cleared = with_tx(&pool, async |tx| {
            let mut repo = PgInvoiceRepository::new(&tx);
            repo.set_internal_note(invoice_id, "   ", sent_at).await?;
            repo.find_by_id(invoice_id).await
        })
        .await
        .unwrap()
        .expect("the invoice");
        assert_eq!(cleared.internal_note, None);

        purge(
            &pool,
            "DELETE FROM invoice_events WHERE org_id = $1",
            org_id.0,
        )
        .await;
        cleanup_with_customer(&pool, org_id, customer_id, owner_id).await;
    }

    /// #342's own invariant: `record_generated_document` succeeds exactly
    /// once against a given invoice, and a second call — even with a
    /// different document — is refused rather than silently replacing what
    /// the first one stored. `trg_invoices_forbid_document_reassignment`
    /// (the migration's own trigger) is the belt behind this method's
    /// `WHERE document_file_key IS NULL`; this test exercises the method,
    /// not the trigger directly, the same way `chk_invoices_issued_state`
    /// is exercised through `issue` rather than with a raw `UPDATE`.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn record_generated_document_is_refused_the_second_time() {
        let pool = dev_pool().await;
        let org_id = seed_organization(&pool, "invoice-document").await;
        let customer_id = seed_customer(&pool, org_id, "invoice-document").await;
        let customer_context_id = seed_customer_context(&pool, customer_id).await;
        let owner_id =
            sqlx::query_scalar!("SELECT owner_id FROM organizations WHERE id = $1", org_id.0)
                .fetch_one(&pool)
                .await
                .unwrap();

        let invoice_id = seed_issued_invoice(
            &pool,
            org_id,
            customer_id,
            customer_context_id,
            InvoiceKind::Standard,
            None,
            None,
            10_000,
            "FAC-2026-9201",
        )
        .await;

        let now = Utc::now().trunc_subsecs(6);
        let first_document = GeneratedInvoiceDocument {
            format: "FACTURX".to_owned(),
            file_key: "invoices/first.pdf".to_owned(),
            mime_type: "application/pdf".to_owned(),
            generated_at: now,
        };

        let updated = with_tx(&pool, async |tx| {
            let mut repo = PgInvoiceRepository::new(&tx);
            repo.record_generated_document(invoice_id, &first_document, now)
                .await
        })
        .await
        .unwrap();

        let stored = updated
            .generated_document
            .expect("the document was just recorded");
        assert_eq!(stored.file_key, "invoices/first.pdf");
        assert_eq!(stored.format, "FACTURX");

        let second_document = GeneratedInvoiceDocument {
            format: "FACTURX".to_owned(),
            file_key: "invoices/second.pdf".to_owned(),
            mime_type: "application/pdf".to_owned(),
            generated_at: now,
        };
        let result = with_tx(&pool, async |tx| {
            let mut repo = PgInvoiceRepository::new(&tx);
            repo.record_generated_document(invoice_id, &second_document, now)
                .await
        })
        .await;

        assert!(
            matches!(result, Err(CoreError::Conflict(_))),
            "a second document must be refused, not silently replace the first: {result:?}"
        );

        // The first document is still exactly what is stored — the refused
        // second call must not have touched the row at all.
        let reloaded = with_tx(&pool, async |tx| {
            let mut repo = PgInvoiceRepository::new(&tx);
            repo.find_by_id(invoice_id).await
        })
        .await
        .unwrap()
        .unwrap();
        assert_eq!(
            reloaded.generated_document.unwrap().file_key,
            "invoices/first.pdf"
        );

        cleanup_with_customer(&pool, org_id, customer_id, owner_id).await;
    }
}

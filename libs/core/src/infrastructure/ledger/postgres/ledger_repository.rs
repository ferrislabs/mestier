use std::str::FromStr;

use chrono::NaiveDate;
use common::CoreError;
use mestier_macros::repository;
use uuid::Uuid;

use crate::{
    CustomerId, DocumentVatRow, InvoiceId, OrganizationId, PaymentMethod, ReceiptRow,
    domain::ledger::ports::LedgerRepository,
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

#[repository(domain = Ledger, backend = Postgres)]
pub struct PgLedgerRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgLedgerRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

struct ReceiptSqlRow {
    id: Uuid,
    date: NaiveDate,
    amount_cents: i64,
    method: String,
    reference: Option<String>,
    note: Option<String>,
    invoice_id: Option<Uuid>,
    invoice_number: Option<String>,
    customer_id: Option<Uuid>,
    customer_name: Option<String>,
}

impl TryFrom<ReceiptSqlRow> for ReceiptRow {
    type Error = CoreError;

    fn try_from(row: ReceiptSqlRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            date: row.date,
            amount_cents: row.amount_cents,
            method: PaymentMethod::from_str(&row.method).map_err(|error| {
                CoreError::Internal(format!("invalid payment method in database: {error}"))
            })?,
            reference: row.reference,
            note: row.note,
            invoice_id: row.invoice_id.map(InvoiceId),
            invoice_number: row.invoice_number,
            customer_id: row.customer_id.map(CustomerId),
            customer_name: row.customer_name,
        })
    }
}

impl<'tx> LedgerRepository for PgLedgerRepository<'tx> {
    async fn receipts(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<ReceiptRow>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            ReceiptSqlRow,
            r#"
            SELECT p.id AS "id!", p.paid_on AS "date!", p.amount_cents::bigint AS "amount_cents!",
                p.method AS "method!", p.reference AS "reference?", p.note AS "note?",
                p.invoice_id AS "invoice_id?", i.number AS "invoice_number?",
                i.customer_id AS "customer_id?", c.name AS "customer_name?"
            FROM invoice_payments p
            JOIN invoices i ON i.id = p.invoice_id AND i.org_id = p.org_id
            JOIN customers c ON c.id = i.customer_id
            WHERE p.org_id = $1 AND p.deleted_at IS NULL AND i.deleted_at IS NULL
            UNION ALL
            SELECT u.id, u.paid_on, u.amount_cents::bigint, u.method, u.reference, u.note,
                NULL::uuid, NULL::text, u.customer_id, c.name
            FROM unallocated_payments u
            LEFT JOIN customers c ON c.id = u.customer_id
            WHERE u.org_id = $1
            "#,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn document_vat(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<DocumentVatRow>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query!(
            r#"
            SELECT issued_on AS "date!", (gross_cents - net_cents)::bigint AS "vat_cents!",
                (kind = 'CREDIT_NOTE') AS "credit_note!"
            FROM invoices
            WHERE org_id = $1 AND deleted_at IS NULL
                AND status NOT IN ('DRAFT', 'CANCELLED') AND issued_on IS NOT NULL
            "#,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows
            .into_iter()
            .map(|row| DocumentVatRow {
                date: row.date,
                vat_cents: row.vat_cents,
                credit_note: row.credit_note,
            })
            .collect())
    }
}

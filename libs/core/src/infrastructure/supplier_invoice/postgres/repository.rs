use common::CoreError;
use mestier_macros::repository;
use sqlx::PgConnection;

use crate::{
    OrganizationId, SupplierInvoice, SupplierInvoiceId, SupplierInvoiceLine, SupplierInvoiceReview,
    domain::supplier_invoice::ports::SupplierInvoiceRepository,
    infrastructure::{
        postgres::{SharedTx, error::map_sqlx_error},
        supplier_invoice::postgres::model::{
            SupplierInvoiceLineRow, SupplierInvoiceRow, vat_breakdown_to_json,
        },
    },
};

#[repository(domain = SupplierInvoice, backend = Postgres)]
pub struct PgSupplierInvoiceRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgSupplierInvoiceRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> SupplierInvoiceRepository for PgSupplierInvoiceRepository<'tx> {
    async fn insert(&mut self, invoice: &SupplierInvoice) -> Result<SupplierInvoice, CoreError> {
        let mut tx = self.tx.lock().await;
        let vat_breakdown = vat_breakdown_to_json(&invoice.vat_breakdown);
        let row = sqlx::query_as!(
            SupplierInvoiceRow,
            r#"
            INSERT INTO supplier_invoices (
                id, org_id, supplier_id, supplier_name, supplier_registration_number,
                supplier_vat_number, number, issued_on, due_on, received_at, source,
                status, currency, notes, net_cents, vat_breakdown, gross_cents,
                deleted_at, created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9, $10, $11,
                CAST($12 AS text)::supplier_invoice_status, $13, $14, $15, $16, $17,
                $18, $19, $20
            )
            RETURNING
                id, org_id, supplier_id, supplier_name, supplier_registration_number,
                supplier_vat_number, number, issued_on, due_on, received_at, source,
                status::text AS "status!", currency, notes, net_cents, vat_breakdown,
                gross_cents, deleted_at, created_at, updated_at
            "#,
            invoice.id.0,
            invoice.organization_id.0,
            invoice.supplier_id.map(|id| id.0),
            invoice.supplier_name,
            invoice.supplier_registration_number,
            invoice.supplier_vat_number,
            invoice.number,
            invoice.issued_on,
            invoice.due_on,
            invoice.received_at,
            invoice.source.as_str(),
            invoice.status.as_str(),
            invoice.currency,
            invoice.notes,
            invoice.net_cents,
            vat_breakdown,
            invoice.gross_cents,
            invoice.deleted_at,
            invoice.created_at,
            invoice.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        insert_lines(&mut tx, &invoice.lines).await?;
        row.into_supplier_invoice(invoice.lines.clone())
    }

    async fn find_by_id(
        &mut self,
        id: SupplierInvoiceId,
    ) -> Result<Option<SupplierInvoice>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            SupplierInvoiceRow,
            r#"
            SELECT
                id, org_id, supplier_id, supplier_name, supplier_registration_number,
                supplier_vat_number, number, issued_on, due_on, received_at, source,
                status::text AS "status!", currency, notes, net_cents, vat_breakdown,
                gross_cents, deleted_at, created_at, updated_at
            FROM supplier_invoices
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
                Ok(Some(row.into_supplier_invoice(lines)?))
            }
            None => Ok(None),
        }
    }

    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<SupplierInvoice>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            SupplierInvoiceRow,
            r#"
            SELECT
                id, org_id, supplier_id, supplier_name, supplier_registration_number,
                supplier_vat_number, number, issued_on, due_on, received_at, source,
                status::text AS "status!", currency, notes, net_cents, vat_breakdown,
                gross_cents, deleted_at, created_at, updated_at
            FROM supplier_invoices
            WHERE org_id = $1 AND deleted_at IS NULL
            ORDER BY received_at DESC, id ASC
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
            r#"SELECT COUNT(*) AS "count!" FROM supplier_invoices WHERE org_id = $1 AND deleted_at IS NULL"#,
            organization_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let mut invoices = Vec::with_capacity(rows.len());
        for row in rows {
            let id = SupplierInvoiceId(row.id);
            let lines = fetch_lines(&mut tx, id).await?;
            invoices.push(row.into_supplier_invoice(lines)?);
        }

        Ok((invoices, total as u64))
    }

    /// Persists only `status` and `notes` — the columns
    /// [`SupplierInvoiceReview`] is allowed to expose a setter for. The
    /// `UPDATE` below naming exactly those two, plus `updated_at`, is what
    /// keeps this true even if a future caller reaches for the struct
    /// directly instead of going through the type.
    async fn update_review(
        &mut self,
        review: &SupplierInvoiceReview,
    ) -> Result<SupplierInvoice, CoreError> {
        let invoice = review.invoice();
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            SupplierInvoiceRow,
            r#"
            UPDATE supplier_invoices
            SET status = CAST($2 AS text)::supplier_invoice_status, notes = $3, updated_at = $4
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING
                id, org_id, supplier_id, supplier_name, supplier_registration_number,
                supplier_vat_number, number, issued_on, due_on, received_at, source,
                status::text AS "status!", currency, notes, net_cents, vat_breakdown,
                gross_cents, deleted_at, created_at, updated_at
            "#,
            invoice.id.0,
            invoice.status.as_str(),
            invoice.notes,
            invoice.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let Some(row) = row else {
            return Err(CoreError::NotFound);
        };
        let lines = fetch_lines(&mut tx, invoice.id).await?;
        row.into_supplier_invoice(lines)
    }

    /// A dynamic (non-`query!`) query, deliberately: #337 lands in the same
    /// branch as a sibling workstream that owns the `.sqlx` offline cache's
    /// stability, and this crate's checkout has no guaranteed live database
    /// to run `cargo sqlx prepare` against for a brand-new query. The
    /// `COALESCE` mirrors [`crate::domain::supplier_invoice::ports::supplier_identifier`]'s
    /// preference order exactly — registration number, then VAT number,
    /// then name — so a row stored under any of the three still matches an
    /// `identifier` computed the same way from a freshly parsed document.
    async fn exists_with_duplicate_key(
        &mut self,
        organization_id: OrganizationId,
        number: &str,
        identifier: &str,
    ) -> Result<bool, CoreError> {
        let mut tx = self.tx.lock().await;
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM supplier_invoices
                WHERE org_id = $1
                  AND number = $2
                  AND deleted_at IS NULL
                  AND COALESCE(
                        NULLIF(TRIM(supplier_registration_number), ''),
                        NULLIF(TRIM(supplier_vat_number), ''),
                        supplier_name
                      ) = $3
            )
            "#,
        )
        .bind(organization_id.0)
        .bind(number)
        .bind(identifier)
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(exists)
    }
}

async fn fetch_lines(
    conn: &mut PgConnection,
    invoice_id: SupplierInvoiceId,
) -> Result<Vec<SupplierInvoiceLine>, CoreError> {
    let rows = sqlx::query_as!(
        SupplierInvoiceLineRow,
        r#"
        SELECT id, org_id, supplier_invoice_id, label, quantity, unit, unit_price_cents,
               line_total_cents, vat_rate_basis_points, position, deleted_at, created_at, updated_at
        FROM supplier_invoice_lines
        WHERE supplier_invoice_id = $1 AND deleted_at IS NULL
        ORDER BY position ASC, id ASC
        "#,
        invoice_id.0,
    )
    .fetch_all(conn)
    .await
    .map_err(map_sqlx_error)?;

    rows.into_iter().map(TryInto::try_into).collect()
}

async fn insert_lines(
    conn: &mut PgConnection,
    lines: &[SupplierInvoiceLine],
) -> Result<(), CoreError> {
    for line in lines {
        sqlx::query!(
            r#"
            INSERT INTO supplier_invoice_lines (
                id, org_id, supplier_invoice_id, label, quantity, unit, unit_price_cents,
                line_total_cents, vat_rate_basis_points, position, deleted_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
            line.id.0,
            line.organization_id.0,
            line.supplier_invoice_id.0,
            line.label,
            line.quantity,
            line.unit,
            line.unit_price_cents,
            line.line_total_cents,
            line.vat_rate_basis_points,
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

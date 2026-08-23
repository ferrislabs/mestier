use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;
use sqlx::PgConnection;

use crate::{
    DraftInvoice, Invoice, InvoiceId, InvoiceLine, InvoiceStatus, LegalIdentity, OrganizationId,
    ProjectId,
    domain::invoice::ports::InvoiceRepository,
    infrastructure::{
        invoice::postgres::model::{
            InvoiceLineRow, InvoiceRow, delivery_address_columns, legal_identity_to_json,
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
        prefix: &str,
        year: i32,
    ) -> Result<String, CoreError> {
        let mut tx = self.tx.lock().await;

        // Same locking device as `PgQuoteRepository::allocate_number`: the
        // `ON CONFLICT DO UPDATE` takes a row lock on the counter for the
        // duration of this transaction, which is what makes two invoices
        // issued at the same instant unable to collide.
        let next_number: i32 = sqlx::query_scalar!(
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
        .map_err(map_sqlx_error)?;

        Ok(format!("{prefix}-{year}-{next_number:04}"))
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
                issuer_identity, deleted_at, created_at, updated_at
            )
            VALUES (
                $1, $2, $3, CAST($4 AS text)::invoice_kind, $5, $6, $7,
                CAST($8 AS text)::invoice_status, $9, $10, $11,
                CAST($12 AS text)::invoice_operation_nature,
                $13, $14, $15, $16, $17,
                $18, $19, $20,
                $21, $22, $23, $24
            )
            RETURNING
                id, org_id, number, kind::text AS "kind!", project_id, customer_id,
                customer_context_id, status::text AS "status!", issued_at, due_at, notes,
                operation_nature::text,
                delivery_address_line1, delivery_address_line2, delivery_address_postal_code,
                delivery_address_city, delivery_address_country,
                net_cents, vat_breakdown, gross_cents, issuer_identity, deleted_at,
                created_at, updated_at
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
                net_cents, vat_breakdown, gross_cents, issuer_identity, deleted_at,
                created_at, updated_at
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
                net_cents, vat_breakdown, gross_cents, issuer_identity, deleted_at,
                created_at, updated_at
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
                net_cents, vat_breakdown, gross_cents, issuer_identity, deleted_at,
                created_at, updated_at
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
                due_at = $5,
                notes = $6,
                operation_nature = CAST($7 AS text)::invoice_operation_nature,
                delivery_address_line1 = $8,
                delivery_address_line2 = $9,
                delivery_address_postal_code = $10,
                delivery_address_city = $11,
                delivery_address_country = $12,
                net_cents = $13,
                vat_breakdown = $14,
                gross_cents = $15,
                updated_at = $16
            WHERE id = $1 AND deleted_at IS NULL AND status = 'DRAFT'
            RETURNING
                id, org_id, number, kind::text AS "kind!", project_id, customer_id,
                customer_context_id, status::text AS "status!", issued_at, due_at, notes,
                operation_nature::text,
                delivery_address_line1, delivery_address_line2, delivery_address_postal_code,
                delivery_address_city, delivery_address_country,
                net_cents, vat_breakdown, gross_cents, issuer_identity, deleted_at,
                created_at, updated_at
            "#,
            invoice.id.0,
            invoice.project_id.map(|id| id.0),
            invoice.customer_id.0,
            invoice.customer_context_id.0,
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
                net_cents, vat_breakdown, gross_cents, issuer_identity, deleted_at,
                created_at, updated_at
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

    async fn issue(
        &mut self,
        id: InvoiceId,
        number: String,
        issued_at: DateTime<Utc>,
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
                issuer_identity = $4,
                updated_at = $5
            WHERE id = $1 AND deleted_at IS NULL AND status = 'DRAFT'
            RETURNING
                id, org_id, number, kind::text AS "kind!", project_id, customer_id,
                customer_context_id, status::text AS "status!", issued_at, due_at, notes,
                operation_nature::text,
                delivery_address_line1, delivery_address_line2, delivery_address_postal_code,
                delivery_address_city, delivery_address_country,
                net_cents, vat_breakdown, gross_cents, issuer_identity, deleted_at,
                created_at, updated_at
            "#,
            id.0,
            number,
            issued_at,
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
}

async fn fetch_lines(
    conn: &mut PgConnection,
    invoice_id: InvoiceId,
) -> Result<Vec<InvoiceLine>, CoreError> {
    let rows = sqlx::query_as!(
        InvoiceLineRow,
        r#"
        SELECT id, org_id, invoice_id, label, quantity, unit_price_cents, vat_rate_basis_points, position, deleted_at, created_at, updated_at
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
                vat_rate_basis_points, position, deleted_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            line.id.0,
            line.organization_id.0,
            line.invoice_id.0,
            line.label,
            line.quantity,
            line.unit_price_cents,
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
            repo.allocate_number(org_id, "FAC", 2026).await
        })
        .await
        .unwrap();
        let second = with_tx(&pool, async |tx| {
            let mut repo = PgInvoiceRepository::new(&tx);
            repo.allocate_number(org_id, "FAC", 2026).await
        })
        .await
        .unwrap();
        let different_year = with_tx(&pool, async |tx| {
            let mut repo = PgInvoiceRepository::new(&tx);
            repo.allocate_number(org_id, "FAC", 2027).await
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
                repo.allocate_number(org_id, "FAC", 2026).await
            }),
            with_tx(&pool_b, async |tx| {
                let mut repo = PgInvoiceRepository::new(&tx);
                repo.allocate_number(org_id, "FAC", 2026).await
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
}

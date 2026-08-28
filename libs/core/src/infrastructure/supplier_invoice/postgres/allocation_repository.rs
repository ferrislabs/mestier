//! Persists [`SupplierInvoiceLineAllocation`] — a table and a repository of
//! its own, separate from [`PgSupplierInvoiceRepository`] one file over: see
//! the doc comment on [`SupplierInvoiceAllocationRepository`] for why.

use common::CoreError;
use mestier_macros::repository;

use crate::{
    OrganizationId, ProjectId, SupplierInvoiceLineAllocation, SupplierInvoiceLineAllocationId,
    SupplierInvoiceLineId,
    domain::supplier_invoice::ports::SupplierInvoiceAllocationRepository,
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

#[repository(domain = SupplierInvoiceAllocation, backend = Postgres)]
pub struct PgSupplierInvoiceAllocationRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgSupplierInvoiceAllocationRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> SupplierInvoiceAllocationRepository for PgSupplierInvoiceAllocationRepository<'tx> {
    async fn insert(
        &mut self,
        allocation: &SupplierInvoiceLineAllocation,
    ) -> Result<SupplierInvoiceLineAllocation, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query!(
            r#"
            INSERT INTO supplier_invoice_line_allocations (
                id, org_id, supplier_invoice_line_id, project_id, amount_cents,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, org_id, supplier_invoice_line_id, project_id, amount_cents,
                      created_at, updated_at
            "#,
            allocation.id.0,
            allocation.organization_id.0,
            allocation.supplier_invoice_line_id.0,
            allocation.project_id.0,
            allocation.amount_cents,
            allocation.created_at,
            allocation.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(SupplierInvoiceLineAllocation {
            id: SupplierInvoiceLineAllocationId(row.id),
            organization_id: OrganizationId(row.org_id),
            supplier_invoice_line_id: SupplierInvoiceLineId(row.supplier_invoice_line_id),
            project_id: ProjectId(row.project_id),
            amount_cents: row.amount_cents,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn sum_allocated_for_line(
        &mut self,
        line_id: SupplierInvoiceLineId,
    ) -> Result<i32, CoreError> {
        let mut tx = self.tx.lock().await;
        let total = sqlx::query_scalar!(
            r#"
            SELECT COALESCE(SUM(amount_cents), 0)::integer AS "total!"
            FROM supplier_invoice_line_allocations
            WHERE supplier_invoice_line_id = $1
            "#,
            line_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(total)
    }

    async fn list_by_project(
        &mut self,
        project_id: ProjectId,
    ) -> Result<Vec<SupplierInvoiceLineAllocation>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query!(
            r#"
            SELECT id, org_id, supplier_invoice_line_id, project_id, amount_cents,
                   created_at, updated_at
            FROM supplier_invoice_line_allocations
            WHERE project_id = $1
            ORDER BY created_at ASC, id ASC
            "#,
            project_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows
            .into_iter()
            .map(|row| SupplierInvoiceLineAllocation {
                id: SupplierInvoiceLineAllocationId(row.id),
                organization_id: OrganizationId(row.org_id),
                supplier_invoice_line_id: SupplierInvoiceLineId(row.supplier_invoice_line_id),
                project_id: ProjectId(row.project_id),
                amount_cents: row.amount_cents,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect())
    }
}

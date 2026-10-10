use common::CoreError;
use mestier_macros::repository;

use crate::{
    OrganizationId, Supplier, SupplierId,
    domain::supplier_invoice::ports::SupplierRepository,
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

struct SupplierRow {
    id: uuid::Uuid,
    org_id: uuid::Uuid,
    name: String,
    registration_number: Option<String>,
    vat_number: Option<String>,
    address: Option<String>,
    phone: Option<String>,
    email: Option<String>,
    deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<SupplierRow> for Supplier {
    fn from(row: SupplierRow) -> Self {
        Self {
            id: SupplierId(row.id),
            organization_id: OrganizationId(row.org_id),
            name: row.name,
            registration_number: row.registration_number,
            vat_number: row.vat_number,
            address: row.address,
            phone: row.phone,
            email: row.email,
            deleted_at: row.deleted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[repository(domain = Supplier, backend = Postgres)]
pub struct PgSupplierRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgSupplierRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> SupplierRepository for PgSupplierRepository<'tx> {
    async fn insert(&mut self, supplier: &Supplier) -> Result<Supplier, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            SupplierRow,
            r#"
            INSERT INTO suppliers (id, org_id, name, registration_number, vat_number, address, phone, email, deleted_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, org_id, name, registration_number, vat_number, address, phone, email, deleted_at, created_at, updated_at
            "#,
            supplier.id.0,
            supplier.organization_id.0,
            supplier.name,
            supplier.registration_number,
            supplier.vat_number,
            supplier.address,
            supplier.phone,
            supplier.email,
            supplier.deleted_at,
            supplier.created_at,
            supplier.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn find_by_id(&mut self, id: SupplierId) -> Result<Option<Supplier>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            SupplierRow,
            r#"
            SELECT id, org_id, name, registration_number, vat_number, address, phone, email, deleted_at, created_at, updated_at
            FROM suppliers
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
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Supplier>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            SupplierRow,
            r#"
            SELECT id, org_id, name, registration_number, vat_number, address, phone, email, deleted_at, created_at, updated_at
            FROM suppliers
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
            r#"SELECT COUNT(*) AS "count!" FROM suppliers WHERE org_id = $1 AND deleted_at IS NULL"#,
            organization_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok((rows.into_iter().map(Into::into).collect(), total as u64))
    }

    async fn update(&mut self, supplier: &Supplier) -> Result<Supplier, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            SupplierRow,
            r#"
            UPDATE suppliers
            SET name = $2, registration_number = $3, vat_number = $4, address = $5, phone = $6, email = $7, updated_at = $8
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, org_id, name, registration_number, vat_number, address, phone, email, deleted_at, created_at, updated_at
            "#,
            supplier.id.0,
            supplier.name,
            supplier.registration_number,
            supplier.vat_number,
            supplier.address,
            supplier.phone,
            supplier.email,
            supplier.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }
}

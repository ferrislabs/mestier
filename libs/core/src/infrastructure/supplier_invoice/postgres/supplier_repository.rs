use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{
    AddressKind, ExpenseCategoryId, OrganizationId, Supplier, SupplierAddress, SupplierContact,
    SupplierId, SupplierProfile,
    domain::supplier_invoice::ports::SupplierRepository,
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

struct SupplierRow {
    id: Uuid,
    org_id: Uuid,
    name: String,
    registration_number: Option<String>,
    vat_number: Option<String>,
    address: Option<String>,
    phone: Option<String>,
    email: Option<String>,
    iban: Option<String>,
    payment_terms_days: Option<i32>,
    default_category_id: Option<Uuid>,
    notes: Option<String>,
    archived_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

struct ContactRow {
    id: Uuid,
    supplier_id: Uuid,
    first_name: String,
    last_name: String,
    role: String,
    email: String,
    phone: String,
    mobile: String,
}

struct AddressRow {
    id: Uuid,
    supplier_id: Uuid,
    kind: String,
    label: String,
    street: String,
    line2: String,
    postal_code: String,
    city: String,
    country: String,
}

struct Children {
    contacts: HashMap<Uuid, Vec<SupplierContact>>,
    addresses: HashMap<Uuid, Vec<SupplierAddress>>,
}

impl SupplierRow {
    fn into_supplier(self, children: &mut Children) -> Supplier {
        Supplier {
            id: SupplierId(self.id),
            organization_id: OrganizationId(self.org_id),
            name: self.name,
            registration_number: self.registration_number,
            vat_number: self.vat_number,
            address: self.address,
            phone: self.phone,
            email: self.email,
            profile: SupplierProfile {
                iban: self.iban,
                payment_terms_days: self.payment_terms_days,
                default_category_id: self.default_category_id.map(ExpenseCategoryId),
                notes: self.notes,
                archived_at: self.archived_at,
                contacts: children.contacts.remove(&self.id).unwrap_or_default(),
                addresses: children.addresses.remove(&self.id).unwrap_or_default(),
            },
            deleted_at: self.deleted_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

async fn load_children(conn: &mut PgConnection, ids: &[Uuid]) -> Result<Children, CoreError> {
    let contacts = sqlx::query_as!(
        ContactRow,
        r#"
        SELECT id, supplier_id, first_name, last_name, role, email, phone, mobile
        FROM supplier_contacts
        WHERE supplier_id = ANY($1)
        ORDER BY position ASC, id ASC
        "#,
        ids,
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(map_sqlx_error)?;
    let addresses = sqlx::query_as!(
        AddressRow,
        r#"
        SELECT id, supplier_id, kind, label, street, line2, postal_code, city, country
        FROM supplier_addresses
        WHERE supplier_id = ANY($1)
        ORDER BY position ASC, id ASC
        "#,
        ids,
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(map_sqlx_error)?;

    let mut children = Children {
        contacts: HashMap::new(),
        addresses: HashMap::new(),
    };
    for row in contacts {
        children
            .contacts
            .entry(row.supplier_id)
            .or_default()
            .push(SupplierContact {
                id: row.id,
                first_name: row.first_name,
                last_name: row.last_name,
                role: row.role,
                email: row.email,
                phone: row.phone,
                mobile: row.mobile,
            });
    }
    for row in addresses {
        let kind = AddressKind::from_str(&row.kind).map_err(|error| {
            CoreError::Internal(format!(
                "invalid supplier address kind in database: {error}"
            ))
        })?;
        children
            .addresses
            .entry(row.supplier_id)
            .or_default()
            .push(SupplierAddress {
                id: row.id,
                kind,
                label: row.label,
                street: row.street,
                line2: row.line2,
                postal_code: row.postal_code,
                city: row.city,
                country: row.country,
            });
    }
    Ok(children)
}

async fn replace_children(conn: &mut PgConnection, supplier: &Supplier) -> Result<(), CoreError> {
    sqlx::query!(
        "DELETE FROM supplier_contacts WHERE supplier_id = $1",
        supplier.id.0
    )
    .execute(&mut *conn)
    .await
    .map_err(map_sqlx_error)?;
    sqlx::query!(
        "DELETE FROM supplier_addresses WHERE supplier_id = $1",
        supplier.id.0
    )
    .execute(&mut *conn)
    .await
    .map_err(map_sqlx_error)?;

    for (position, contact) in supplier.profile.contacts.iter().enumerate() {
        sqlx::query!(
            r#"
            INSERT INTO supplier_contacts (id, org_id, supplier_id, position, first_name, last_name, role, email, phone, mobile)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            contact.id,
            supplier.organization_id.0,
            supplier.id.0,
            position as i32,
            contact.first_name,
            contact.last_name,
            contact.role,
            contact.email,
            contact.phone,
            contact.mobile,
        )
        .execute(&mut *conn)
        .await
        .map_err(map_sqlx_error)?;
    }
    for (position, address) in supplier.profile.addresses.iter().enumerate() {
        sqlx::query!(
            r#"
            INSERT INTO supplier_addresses (id, org_id, supplier_id, position, kind, label, street, line2, postal_code, city, country)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            address.id,
            supplier.organization_id.0,
            supplier.id.0,
            position as i32,
            address.kind.as_str(),
            address.label,
            address.street,
            address.line2,
            address.postal_code,
            address.city,
            address.country,
        )
        .execute(&mut *conn)
        .await
        .map_err(map_sqlx_error)?;
    }
    Ok(())
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
        {
            let mut tx = self.tx.lock().await;
            sqlx::query!(
                r#"
                INSERT INTO suppliers (id, org_id, name, registration_number, vat_number, address, phone, email, iban, payment_terms_days, default_category_id, notes, archived_at, deleted_at, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
                "#,
                supplier.id.0,
                supplier.organization_id.0,
                supplier.name,
                supplier.registration_number,
                supplier.vat_number,
                supplier.address,
                supplier.phone,
                supplier.email,
                supplier.profile.iban,
                supplier.profile.payment_terms_days,
                supplier.profile.default_category_id.map(|id| id.0),
                supplier.profile.notes,
                supplier.profile.archived_at,
                supplier.deleted_at,
                supplier.created_at,
                supplier.updated_at,
            )
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
            replace_children(&mut tx, supplier).await?;
        }

        self.find_by_id(supplier.id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    async fn find_by_id(&mut self, id: SupplierId) -> Result<Option<Supplier>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            SupplierRow,
            r#"
            SELECT id, org_id, name, registration_number, vat_number, address, phone, email, iban, payment_terms_days, default_category_id, notes, archived_at, deleted_at, created_at, updated_at
            FROM suppliers
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        match row {
            Some(row) => {
                let mut children = load_children(&mut tx, &[row.id]).await?;
                Ok(Some(row.into_supplier(&mut children)))
            }
            None => Ok(None),
        }
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
            SELECT id, org_id, name, registration_number, vat_number, address, phone, email, iban, payment_terms_days, default_category_id, notes, archived_at, deleted_at, created_at, updated_at
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

        let ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
        let mut children = load_children(&mut tx, &ids).await?;
        let suppliers = rows
            .into_iter()
            .map(|row| row.into_supplier(&mut children))
            .collect();

        Ok((suppliers, total as u64))
    }

    async fn update(&mut self, supplier: &Supplier) -> Result<Supplier, CoreError> {
        {
            let mut tx = self.tx.lock().await;
            let result = sqlx::query!(
                r#"
                UPDATE suppliers
                SET name = $2, registration_number = $3, vat_number = $4, address = $5, phone = $6, email = $7, iban = $8, payment_terms_days = $9, default_category_id = $10, notes = $11, updated_at = $12
                WHERE id = $1 AND deleted_at IS NULL
                "#,
                supplier.id.0,
                supplier.name,
                supplier.registration_number,
                supplier.vat_number,
                supplier.address,
                supplier.phone,
                supplier.email,
                supplier.profile.iban,
                supplier.profile.payment_terms_days,
                supplier.profile.default_category_id.map(|id| id.0),
                supplier.profile.notes,
                supplier.updated_at,
            )
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;

            if result.rows_affected() == 0 {
                return Err(CoreError::NotFound);
            }
            replace_children(&mut tx, supplier).await?;
        }

        self.find_by_id(supplier.id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    async fn set_archived(
        &mut self,
        id: SupplierId,
        archived_at: Option<DateTime<Utc>>,
        updated_at: DateTime<Utc>,
    ) -> Result<Supplier, CoreError> {
        {
            let mut tx = self.tx.lock().await;
            let result = sqlx::query!(
                r#"
                UPDATE suppliers
                SET archived_at = $2, updated_at = $3
                WHERE id = $1 AND deleted_at IS NULL
                "#,
                id.0,
                archived_at,
                updated_at,
            )
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;

            if result.rows_affected() == 0 {
                return Err(CoreError::NotFound);
            }
        }

        self.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }
}

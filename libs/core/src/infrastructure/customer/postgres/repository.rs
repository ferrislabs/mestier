use std::str::FromStr;

use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    Customer, CustomerContactId, CustomerContextId, CustomerDirectoryEntry, CustomerId,
    CustomerInvoiceFact, InvoiceKind, InvoiceStatus, LeadSource, OrganizationId, PaymentTermsDays,
    PrimaryContact,
    domain::customer::ports::CustomerRepository,
    infrastructure::{
        customer::postgres::model::CustomerRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = Customer, backend = Postgres)]
pub struct PgCustomerRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgCustomerRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> CustomerRepository for PgCustomerRepository<'tx> {
    async fn insert(&mut self, customer: &Customer) -> Result<Customer, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CustomerRow,
            r#"
            INSERT INTO customers (id, org_id, status, name, registration_number, phone, email, kind, website, vat_number, owner_member_id, lead_source, notes, payment_terms_days, billing_contact_id, billing_address_id, converted_at, archived_at, deleted_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
            RETURNING id, org_id, status, name, registration_number, phone, email, kind, website, vat_number, owner_member_id, lead_source, notes, payment_terms_days, billing_contact_id, billing_address_id, converted_at, archived_at, deleted_at, created_at, updated_at
            "#,
            customer.id.0,
            customer.organization_id.0,
            customer.status.as_str(),
            customer.name,
            customer.registration_number,
            customer.phone,
            customer.email,
            customer.profile.kind.as_str(),
            customer.profile.website,
            customer.profile.vat_number,
            customer.profile.owner_member_id.map(|id| id.0),
            customer.profile.lead_source.map(LeadSource::as_str),
            customer.profile.notes,
            customer.profile.payment_terms.map(PaymentTermsDays::get),
            customer.profile.billing_contact_id.map(|id| id.0),
            customer.profile.billing_address_id.map(|id| id.0),
            customer.converted_at,
            customer.archived_at,
            customer.deleted_at,
            customer.created_at,
            customer.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.try_into()
    }

    async fn find_by_id(&mut self, id: CustomerId) -> Result<Option<Customer>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CustomerRow,
            r#"
            SELECT id, org_id, status, name, registration_number, phone, email, kind, website, vat_number, owner_member_id, lead_source, notes, payment_terms_days, billing_contact_id, billing_address_id, converted_at, archived_at, deleted_at, created_at, updated_at
            FROM customers
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
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Customer>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            CustomerRow,
            r#"
            SELECT id, org_id, status, name, registration_number, phone, email, kind, website, vat_number, owner_member_id, lead_source, notes, payment_terms_days, billing_contact_id, billing_address_id, converted_at, archived_at, deleted_at, created_at, updated_at
            FROM customers
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
            r#"SELECT COUNT(*) AS "count!" FROM customers WHERE org_id = $1 AND deleted_at IS NULL"#,
            organization_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let customers = rows
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, CoreError>>()?;

        Ok((customers, total as u64))
    }

    async fn directory(
        &mut self,
        organization_id: OrganizationId,
        customer_ids: &[CustomerId],
    ) -> Result<Vec<CustomerDirectoryEntry>, CoreError> {
        let mut tx = self.tx.lock().await;
        let ids: Vec<uuid::Uuid> = customer_ids.iter().map(|id| id.0).collect();
        let rows = sqlx::query!(
            r#"
            SELECT
                c.id AS "customer_id!",
                (
                    SELECT NULLIF(btrim(x.city), '')
                    FROM customer_contexts x
                    WHERE x.customer_id = c.id AND x.deleted_at IS NULL
                    ORDER BY (x.id IS NOT DISTINCT FROM c.billing_address_id) DESC,
                             (x.kind = 'BILLING') DESC, x.created_at ASC, x.id ASC
                    LIMIT 1
                ) AS "city?",
                k.first_name AS "first_name?",
                k.last_name AS "last_name?",
                k.role AS "role?",
                k.email AS "email?",
                k.phone AS "phone?",
                k.mobile AS "mobile?",
                (
                    SELECT MAX(q.created_at)
                    FROM quotes q
                    WHERE q.customer_id = c.id AND q.org_id = c.org_id AND q.deleted_at IS NULL
                ) AS "last_quote_at?"
            FROM customers c
            LEFT JOIN LATERAL (
                SELECT first_name, last_name, role, email, phone, mobile
                FROM customer_contacts
                WHERE customer_id = c.id AND deleted_at IS NULL
                ORDER BY is_primary DESC, created_at ASC, id ASC
                LIMIT 1
            ) k ON TRUE
            WHERE c.org_id = $1 AND c.id = ANY($2) AND c.deleted_at IS NULL
            "#,
            organization_id.0,
            &ids,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows
            .into_iter()
            .map(|row| CustomerDirectoryEntry {
                customer_id: CustomerId(row.customer_id),
                city: row.city,
                primary_contact: row.first_name.zip(row.last_name).map(
                    |(first_name, last_name)| PrimaryContact {
                        first_name,
                        last_name,
                        role: row.role,
                        email: row.email,
                        phone: row.phone,
                        mobile: row.mobile,
                    },
                ),
                last_quote_at: row.last_quote_at,
            })
            .collect())
    }

    async fn invoice_facts(
        &mut self,
        organization_id: OrganizationId,
        customer_ids: &[CustomerId],
    ) -> Result<Vec<CustomerInvoiceFact>, CoreError> {
        let mut tx = self.tx.lock().await;
        let ids: Vec<uuid::Uuid> = customer_ids.iter().map(|id| id.0).collect();
        let rows = sqlx::query!(
            r#"
            SELECT
                i.customer_id AS "customer_id!",
                i.kind::text AS "kind!",
                i.status::text AS "status!",
                i.gross_cents AS "gross_cents!",
                i.net_cents AS "net_cents!",
                i.issued_at AS "issued_at!",
                i.issued_on AS "issued_on!",
                i.due_at AS "due_at?",
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
            WHERE i.org_id = $1 AND i.customer_id = ANY($2) AND i.deleted_at IS NULL
                AND i.status NOT IN ('DRAFT', 'CANCELLED')
            "#,
            organization_id.0,
            &ids,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter()
            .map(|row| {
                Ok(CustomerInvoiceFact {
                    customer_id: CustomerId(row.customer_id),
                    kind: InvoiceKind::from_str(&row.kind).map_err(|e| {
                        CoreError::Internal(format!("invalid invoice kind in database: {e}"))
                    })?,
                    status: InvoiceStatus::from_str(&row.status).map_err(|e| {
                        CoreError::Internal(format!("invalid invoice status in database: {e}"))
                    })?,
                    gross_cents: row.gross_cents,
                    net_cents: row.net_cents,
                    issued_at: row.issued_at,
                    issued_on: row.issued_on,
                    due_on: row.due_at.map(|due| due.date_naive()),
                    paid_cents: row.paid_cents,
                    credited_cents: row.credited_cents,
                })
            })
            .collect()
    }

    async fn update(&mut self, customer: &Customer) -> Result<Customer, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CustomerRow,
            r#"
            UPDATE customers
            SET status = $2, name = $3, registration_number = $4, phone = $5, email = $6,
                kind = $7, website = $8, vat_number = $9, owner_member_id = $10, lead_source = $11, notes = $12,
                payment_terms_days = $13, billing_contact_id = $14, billing_address_id = $15,
                converted_at = $16, archived_at = $17, updated_at = $18
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, org_id, status, name, registration_number, phone, email, kind, website, vat_number, owner_member_id, lead_source, notes, payment_terms_days, billing_contact_id, billing_address_id, converted_at, archived_at, deleted_at, created_at, updated_at
            "#,
            customer.id.0,
            customer.status.as_str(),
            customer.name,
            customer.registration_number,
            customer.phone,
            customer.email,
            customer.profile.kind.as_str(),
            customer.profile.website,
            customer.profile.vat_number,
            customer.profile.owner_member_id.map(|id| id.0),
            customer.profile.lead_source.map(LeadSource::as_str),
            customer.profile.notes,
            customer.profile.payment_terms.map(PaymentTermsDays::get),
            customer.profile.billing_contact_id.map(|id| id.0),
            customer.profile.billing_address_id.map(|id| id.0),
            customer.converted_at,
            customer.archived_at,
            customer.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(TryInto::try_into)
            .transpose()?
            .ok_or(CoreError::NotFound)
    }

    async fn has_contact(
        &mut self,
        customer_id: CustomerId,
        contact_id: CustomerContactId,
    ) -> Result<bool, CoreError> {
        let mut tx = self.tx.lock().await;
        let found = sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM customer_contacts
                WHERE id = $1 AND customer_id = $2 AND deleted_at IS NULL
            ) AS "found!"
            "#,
            contact_id.0,
            customer_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(found)
    }

    async fn has_address(
        &mut self,
        customer_id: CustomerId,
        address_id: CustomerContextId,
    ) -> Result<bool, CoreError> {
        let mut tx = self.tx.lock().await;
        let found = sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM customer_contexts
                WHERE id = $1 AND customer_id = $2 AND deleted_at IS NULL
            ) AS "found!"
            "#,
            address_id.0,
            customer_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(found)
    }

    async fn count_converted_since(
        &mut self,
        organization_id: OrganizationId,
        since: DateTime<Utc>,
    ) -> Result<i64, CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM customers
            WHERE org_id = $1 AND deleted_at IS NULL AND converted_at >= $2
            "#,
            organization_id.0,
            since,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)
    }

    async fn soft_delete(
        &mut self,
        id: CustomerId,
        deleted_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE customers
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
            UPDATE customer_contexts
            SET deleted_at = $2, updated_at = $2
            WHERE customer_id = $1 AND deleted_at IS NULL
            "#,
            id.0,
            deleted_at,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        sqlx::query!(
            r#"
            UPDATE customer_contacts
            SET deleted_at = $2, updated_at = $2
            WHERE customer_id = $1 AND deleted_at IS NULL
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

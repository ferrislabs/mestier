use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;
use tracing::info;

use crate::{
    UserId,
    domain::organization::{Organization, OrganizationId, ports::OrganizationRepository},
    infrastructure::{
        organization::postgres::model::{OrganizationRow, vat_status_columns},
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

// Every query below spells out the same organization column list. `query_as!`
// needs the literal SQL to verify at compile time, so it cannot be
// deduplicated into a constant; `OrganizationRow` maps columns positionally,
// so keep the list identical (and in the same order) everywhere it appears.

#[repository(domain = Organization, backend = Postgres)]
pub struct PgOrganizationRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgOrganizationRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> OrganizationRepository for PgOrganizationRepository<'tx> {
    #[tracing::instrument(skip(self, organization), fields(db.system = "postgresql", db.operation = "insert", db.table = "organizations", organization.slug = %organization.slug), err)]
    async fn insert(&mut self, organization: &Organization) -> Result<Organization, CoreError> {
        let mut tx = self.tx.lock().await;
        let (vat_status, vat_number, vat_exemption_basis) =
            vat_status_columns(&organization.vat_status);
        let row = sqlx::query_as!(
            OrganizationRow,
            r#"
            INSERT INTO organizations (
                id, name, slug, owner_id,
                legal_name, legal_form, registration_number,
                vat_status, vat_number, vat_exemption_basis,
                share_capital_cents,
                address_line1, address_line2, address_postal_code, address_city, address_country,
                contact_email, contact_phone, insurance_mention,
                field_clock_enabled,
                vat_on_debits,
                deleted_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24)
            RETURNING
                id, name, slug, owner_id,
                legal_name, legal_form, registration_number,
                vat_status, vat_number, vat_exemption_basis,
                share_capital_cents,
                address_line1, address_line2, address_postal_code, address_city, address_country,
                contact_email, contact_phone, insurance_mention,
                quote_number_prefix,
                invoice_number_prefix,
                field_clock_enabled,
                vat_on_debits,
                deleted_at, created_at, updated_at
            "#,
            organization.id.0,
            organization.name,
            organization.slug,
            organization.owner_id.0,
            organization.legal_name,
            organization.legal_form,
            organization.registration_number,
            vat_status,
            vat_number,
            vat_exemption_basis,
            organization.share_capital_cents,
            organization.address_line1,
            organization.address_line2,
            organization.address_postal_code,
            organization.address_city,
            organization.address_country,
            organization.contact_email,
            organization.contact_phone,
            organization.insurance_mention,
            organization.field_clock_enabled,
            organization.vat_on_debits,
            organization.deleted_at,
            organization.created_at,
            organization.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.into_organization()
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "organizations"), err)]
    async fn find_by_id(&mut self, id: OrganizationId) -> Result<Option<Organization>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            OrganizationRow,
            r#"
            SELECT id, name, slug, owner_id,
                legal_name, legal_form, registration_number,
                vat_status, vat_number, vat_exemption_basis,
                share_capital_cents,
                address_line1, address_line2, address_postal_code, address_city, address_country,
                contact_email, contact_phone, insurance_mention,
                quote_number_prefix,
                invoice_number_prefix,
                field_clock_enabled,
                vat_on_debits,
                deleted_at, created_at, updated_at
            FROM organizations
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(OrganizationRow::into_organization).transpose()
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "organizations"), err)]
    async fn find_timezone(&mut self, id: OrganizationId) -> Result<Option<String>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query!(
            r#"SELECT timezone FROM organizations WHERE id = $1 AND deleted_at IS NULL"#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(|row| row.timezone))
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "organizations"), err)]
    async fn list_for_user(&mut self, user_id: UserId) -> Result<Vec<Organization>, CoreError> {
        let mut tx = self.tx.lock().await;
        info!("user_id: {:?}", user_id);
        let rows = sqlx::query_as!(
            OrganizationRow,
            r#"
            SELECT o.id, o.name, o.slug, o.owner_id,
                o.legal_name, o.legal_form, o.registration_number,
                o.vat_status, o.vat_number, o.vat_exemption_basis,
                o.share_capital_cents,
                o.address_line1, o.address_line2, o.address_postal_code, o.address_city, o.address_country,
                o.contact_email, o.contact_phone, o.insurance_mention,
                o.quote_number_prefix,
                o.invoice_number_prefix,
                o.field_clock_enabled,
                o.vat_on_debits,
                o.deleted_at, o.created_at, o.updated_at
            FROM organizations o
            INNER JOIN organization_members m ON m.organization_id = o.id
            WHERE m.user_id = $1 AND o.deleted_at IS NULL
            ORDER BY o.created_at ASC
            "#,
            user_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter()
            .map(OrganizationRow::into_organization)
            .collect()
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "organizations"), err)]
    async fn list_paginated(
        &mut self,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Organization>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            OrganizationRow,
            r#"
            SELECT id, name, slug, owner_id,
                legal_name, legal_form, registration_number,
                vat_status, vat_number, vat_exemption_basis,
                share_capital_cents,
                address_line1, address_line2, address_postal_code, address_city, address_country,
                contact_email, contact_phone, insurance_mention,
                quote_number_prefix,
                invoice_number_prefix,
                field_clock_enabled,
                vat_on_debits,
                deleted_at, created_at, updated_at
            FROM organizations
            WHERE deleted_at IS NULL
            ORDER BY created_at ASC
            LIMIT $1 OFFSET $2
            "#,
            limit as i64,
            offset as i64,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM organizations WHERE deleted_at IS NULL"#
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let organizations: Result<Vec<Organization>, CoreError> = rows
            .into_iter()
            .map(OrganizationRow::into_organization)
            .collect();

        Ok((organizations?, total as u64))
    }

    #[tracing::instrument(skip(self, organization), fields(db.system = "postgresql", db.operation = "update", db.table = "organizations"), err)]
    async fn update(&mut self, organization: &Organization) -> Result<Organization, CoreError> {
        let mut tx = self.tx.lock().await;
        let (vat_status, vat_number, vat_exemption_basis) =
            vat_status_columns(&organization.vat_status);
        let row = sqlx::query_as!(
            OrganizationRow,
            r#"
            UPDATE organizations
            SET name = $2,
                slug = $3,
                legal_name = $4,
                legal_form = $5,
                registration_number = $6,
                vat_status = $7,
                vat_number = $8,
                vat_exemption_basis = $9,
                share_capital_cents = $10,
                address_line1 = $11,
                address_line2 = $12,
                address_postal_code = $13,
                address_city = $14,
                address_country = $15,
                contact_email = $16,
                contact_phone = $17,
                insurance_mention = $18,
                field_clock_enabled = $19,
                vat_on_debits = $20,
                updated_at = $21
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING
                id, name, slug, owner_id,
                legal_name, legal_form, registration_number,
                vat_status, vat_number, vat_exemption_basis,
                share_capital_cents,
                address_line1, address_line2, address_postal_code, address_city, address_country,
                contact_email, contact_phone, insurance_mention,
                quote_number_prefix,
                invoice_number_prefix,
                field_clock_enabled,
                vat_on_debits,
                deleted_at, created_at, updated_at
            "#,
            organization.id.0,
            organization.name,
            organization.slug,
            organization.legal_name,
            organization.legal_form,
            organization.registration_number,
            vat_status,
            vat_number,
            vat_exemption_basis,
            organization.share_capital_cents,
            organization.address_line1,
            organization.address_line2,
            organization.address_postal_code,
            organization.address_city,
            organization.address_country,
            organization.contact_email,
            organization.contact_phone,
            organization.insurance_mention,
            organization.field_clock_enabled,
            organization.vat_on_debits,
            organization.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        match row {
            Some(row) => row.into_organization(),
            None => Err(CoreError::NotFound),
        }
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "update", db.table = "organizations"), err)]
    async fn soft_delete(
        &mut self,
        id: OrganizationId,
        deleted_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE organizations
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

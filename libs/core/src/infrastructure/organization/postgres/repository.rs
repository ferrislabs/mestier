use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;
use tracing::info;

use crate::{
    UserId,
    domain::organization::{
        Organization, OrganizationId, billing_settings::BillingSettings,
        ports::OrganizationRepository,
    },
    infrastructure::{
        organization::postgres::model::{BillingSettingsRow, OrganizationRow, vat_status_columns},
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
                mcp_enabled,
                deleted_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25)
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
                mcp_enabled,
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
            organization.mcp_enabled,
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
                mcp_enabled,
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
                o.mcp_enabled,
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
                mcp_enabled,
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
                mcp_enabled = $21,
                updated_at = $22
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
                mcp_enabled,
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
            organization.mcp_enabled,
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

    async fn find_billing_settings(
        &mut self,
        id: OrganizationId,
    ) -> Result<Option<BillingSettings>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            BillingSettingsRow,
            r#"
            SELECT
                payment_terms_days, quote_validity_days, default_vat_rate_bp, late_penalty,
                recovery_fee_cents, invoice_footer, number_padding, iban, bic
            FROM organization_billing_settings
            WHERE org_id = $1
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(BillingSettings::try_from).transpose()
    }

    async fn upsert_billing_settings(
        &mut self,
        id: OrganizationId,
        settings: &BillingSettings,
    ) -> Result<BillingSettings, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            BillingSettingsRow,
            r#"
            INSERT INTO organization_billing_settings (
                org_id, payment_terms_days, quote_validity_days, default_vat_rate_bp,
                late_penalty, recovery_fee_cents, invoice_footer, number_padding, iban, bic
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (org_id) DO UPDATE SET
                payment_terms_days = EXCLUDED.payment_terms_days,
                quote_validity_days = EXCLUDED.quote_validity_days,
                default_vat_rate_bp = EXCLUDED.default_vat_rate_bp,
                late_penalty = EXCLUDED.late_penalty,
                recovery_fee_cents = EXCLUDED.recovery_fee_cents,
                invoice_footer = EXCLUDED.invoice_footer,
                number_padding = EXCLUDED.number_padding,
                iban = EXCLUDED.iban,
                bic = EXCLUDED.bic,
                updated_at = now()
            RETURNING
                payment_terms_days, quote_validity_days, default_vat_rate_bp, late_penalty,
                recovery_fee_cents, invoice_footer, number_padding, iban, bic
            "#,
            id.0,
            settings.payment_terms_days.get(),
            settings.quote_validity_days.get(),
            settings.default_vat_rate.basis_points(),
            settings.late_penalty.as_str(),
            settings.recovery_fee.get(),
            settings.invoice_footer,
            settings.number_padding.as_i32(),
            settings.iban,
            settings.bic,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        BillingSettings::try_from(row)
    }
}

#[cfg(test)]
mod tests {
    use common::generate_uuid_v7;
    use sqlx::PgPool;

    use super::*;
    use crate::{
        application::test_support::{dev_pool, purge},
        domain::organization::billing_settings::BillingSettingsInput,
        infrastructure::postgres::with_tx,
    };

    async fn seed_organization(pool: &PgPool, label: &str) -> (OrganizationId, uuid::Uuid) {
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

        (OrganizationId(org_id), owner_id)
    }

    async fn cleanup(pool: &PgPool, org_id: OrganizationId, owner_id: uuid::Uuid) {
        purge(
            pool,
            "DELETE FROM organization_billing_settings WHERE org_id = $1",
            org_id.0,
        )
        .await;
        purge(pool, "DELETE FROM organizations WHERE id = $1", org_id.0).await;
        purge(pool, "DELETE FROM users WHERE id = $1", owner_id).await;
    }

    fn settings(padding: i64, iban: Option<&str>) -> BillingSettings {
        BillingSettings::try_from(BillingSettingsInput {
            payment_terms_days: 45,
            quote_validity_days: 15,
            default_vat_rate_bp: 1000,
            late_penalty: "dix pour cent".to_owned(),
            recovery_fee_cents: 5000,
            invoice_footer: "Merci.".to_owned(),
            number_padding: padding,
            iban: iban.map(str::to_owned),
            bic: Some("AGRIFRPP".to_owned()),
        })
        .unwrap()
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn billing_settings_are_absent_until_saved_then_replaced_whole_and_scoped_by_organization()
     {
        let pool = dev_pool().await;
        let (org_id, owner_id) = seed_organization(&pool, "billing-settings").await;
        let (other_id, other_owner) = seed_organization(&pool, "billing-settings-other").await;

        let (before, first, replaced, other) = with_tx(&pool, async |tx| {
            let mut repo = PgOrganizationRepository::new(&tx);
            let before = repo.find_billing_settings(org_id).await?;
            let first = repo
                .upsert_billing_settings(org_id, &settings(5, Some("FR76 3000")))
                .await?;
            let replaced = repo
                .upsert_billing_settings(org_id, &settings(3, None))
                .await?;
            let other = repo.find_billing_settings(other_id).await?;
            Ok((before, first, replaced, other))
        })
        .await
        .unwrap();

        assert_eq!(before, None);
        assert_eq!(first, settings(5, Some("FR76 3000")));
        assert_eq!(replaced, settings(3, None));
        assert_eq!(other, None);

        cleanup(&pool, org_id, owner_id).await;
        cleanup(&pool, other_id, other_owner).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_database_refuses_settings_out_of_bounds() {
        let pool = dev_pool().await;
        let (org_id, owner_id) = seed_organization(&pool, "billing-bounds").await;

        let payment_terms = sqlx::query(
            "INSERT INTO organization_billing_settings (org_id, payment_terms_days) VALUES ($1, 61)",
        )
        .bind(org_id.0)
        .execute(&pool)
        .await;
        let padding = sqlx::query(
            "INSERT INTO organization_billing_settings (org_id, number_padding) VALUES ($1, 7)",
        )
        .bind(org_id.0)
        .execute(&pool)
        .await;
        let validity = sqlx::query(
            "INSERT INTO organization_billing_settings (org_id, quote_validity_days) VALUES ($1, 0)",
        )
        .bind(org_id.0)
        .execute(&pool)
        .await;
        let fee = sqlx::query(
            "INSERT INTO organization_billing_settings (org_id, recovery_fee_cents) VALUES ($1, -1)",
        )
        .bind(org_id.0)
        .execute(&pool)
        .await;

        assert!(payment_terms.is_err());
        assert!(padding.is_err());
        assert!(validity.is_err());
        assert!(fee.is_err());

        cleanup(&pool, org_id, owner_id).await;
    }
}

use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    HostCompany, HostCompanyId, OrganizationId,
    domain::cursus_host_company::ports::CursusHostCompanyRepository,
    infrastructure::{
        cursus_host_company::postgres::model::HostCompanyRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = CursusHostCompany, backend = Postgres)]
pub struct PgCursusHostCompanyRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgCursusHostCompanyRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> CursusHostCompanyRepository for PgCursusHostCompanyRepository<'tx> {
    async fn insert(&mut self, company: &HostCompany) -> Result<HostCompany, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            HostCompanyRow,
            r#"
            INSERT INTO cursus.host_company
                (id, org_id, name, siret, address, legal_form, naf_code, employee_count, director_name, archived_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, org_id, name, siret, address, legal_form, naf_code, employee_count, director_name, archived_at, created_at, updated_at
            "#,
            company.id.0,
            company.organization_id.0,
            company.name,
            company.siret,
            company.address,
            company.legal_form,
            company.naf_code,
            company.employee_count,
            company.director_name,
            company.archived_at,
            company.created_at,
            company.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn find(
        &mut self,
        organization_id: OrganizationId,
        id: HostCompanyId,
    ) -> Result<Option<HostCompany>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            HostCompanyRow,
            r#"
            SELECT id, org_id, name, siret, address, legal_form, naf_code, employee_count, director_name, archived_at, created_at, updated_at
            FROM cursus.host_company
            WHERE org_id = $1 AND id = $2
            "#,
            organization_id.0,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    async fn list(
        &mut self,
        organization_id: OrganizationId,
        include_archived: bool,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<HostCompany>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            HostCompanyRow,
            r#"
            SELECT id, org_id, name, siret, address, legal_form, naf_code, employee_count, director_name, archived_at, created_at, updated_at
            FROM cursus.host_company
            WHERE org_id = $1 AND ($2 OR archived_at IS NULL)
            ORDER BY name ASC, created_at ASC
            LIMIT $3 OFFSET $4
            "#,
            organization_id.0,
            include_archived,
            limit as i64,
            offset as i64,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let total: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM cursus.host_company
            WHERE org_id = $1 AND ($2 OR archived_at IS NULL)
            "#,
            organization_id.0,
            include_archived,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok((rows.into_iter().map(Into::into).collect(), total as u64))
    }

    async fn update(&mut self, company: &HostCompany) -> Result<HostCompany, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            HostCompanyRow,
            r#"
            UPDATE cursus.host_company
            SET name = $3, siret = $4, address = $5, legal_form = $6, naf_code = $7,
                employee_count = $8, director_name = $9, updated_at = $10
            WHERE org_id = $1 AND id = $2
            RETURNING id, org_id, name, siret, address, legal_form, naf_code, employee_count, director_name, archived_at, created_at, updated_at
            "#,
            company.organization_id.0,
            company.id.0,
            company.name,
            company.siret,
            company.address,
            company.legal_form,
            company.naf_code,
            company.employee_count,
            company.director_name,
            company.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }

    async fn archive(
        &mut self,
        organization_id: OrganizationId,
        id: HostCompanyId,
        archived_at: DateTime<Utc>,
    ) -> Result<HostCompany, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            HostCompanyRow,
            r#"
            UPDATE cursus.host_company
            SET archived_at = COALESCE(archived_at, $3), updated_at = $3
            WHERE org_id = $1 AND id = $2
            RETURNING id, org_id, name, siret, address, legal_form, naf_code, employee_count, director_name, archived_at, created_at, updated_at
            "#,
            organization_id.0,
            id.0,
            archived_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }
}

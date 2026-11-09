use chrono::{DateTime, Utc};
use common::{CoreError, generate_uuid_v7};
use mestier_macros::repository;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{
    HostCompany, HostCompanyId, OrganizationId,
    domain::cursus_host_company::ports::CursusHostCompanyRepository,
    infrastructure::{
        cursus_host_company::postgres::model::{HostCompanyAddressRow, HostCompanyRow, assemble},
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

async fn load_addresses(
    conn: &mut PgConnection,
    organization_id: OrganizationId,
    company_ids: &[Uuid],
) -> Result<Vec<HostCompanyAddressRow>, CoreError> {
    sqlx::query_as!(
        HostCompanyAddressRow,
        r#"
        SELECT host_company_id, label, line1, line2, postal_code, city, is_main
        FROM cursus.host_company_address
        WHERE org_id = $1 AND host_company_id = ANY($2)
        ORDER BY host_company_id, position ASC
        "#,
        organization_id.0,
        company_ids,
    )
    .fetch_all(conn)
    .await
    .map_err(map_sqlx_error)
}

async fn replace_addresses(
    conn: &mut PgConnection,
    company: &HostCompany,
) -> Result<(), CoreError> {
    sqlx::query!(
        "DELETE FROM cursus.host_company_address WHERE org_id = $1 AND host_company_id = $2",
        company.organization_id.0,
        company.id.0,
    )
    .execute(&mut *conn)
    .await
    .map_err(map_sqlx_error)?;

    for (position, address) in company.addresses.iter().enumerate() {
        sqlx::query!(
            r#"
            INSERT INTO cursus.host_company_address
                (id, org_id, host_company_id, label, line1, line2, postal_code, city, is_main, position)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            generate_uuid_v7(),
            company.organization_id.0,
            company.id.0,
            address.label,
            address.line1,
            address.line2,
            address.postal_code,
            address.city,
            address.is_main,
            position as i32,
        )
        .execute(&mut *conn)
        .await
        .map_err(map_sqlx_error)?;
    }

    Ok(())
}

async fn hydrate(conn: &mut PgConnection, row: HostCompanyRow) -> Result<HostCompany, CoreError> {
    let addresses = load_addresses(conn, OrganizationId(row.org_id), &[row.id]).await?;

    Ok(assemble(vec![row], addresses).remove(0))
}

impl<'tx> CursusHostCompanyRepository for PgCursusHostCompanyRepository<'tx> {
    async fn insert(&mut self, company: &HostCompany) -> Result<HostCompany, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            HostCompanyRow,
            r#"
            INSERT INTO cursus.host_company
                (id, org_id, name, group_name, siret, legal_form, naf_code, capital, employee_count, group_employee_count, director_name, phone, email, website, activity_domains, missions, archived_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            RETURNING id, org_id, name, group_name, siret, legal_form, naf_code, capital, employee_count, group_employee_count, director_name, phone, email, website, activity_domains, missions, archived_at, created_at, updated_at
            "#,
            company.id.0,
            company.organization_id.0,
            company.name,
            company.group_name,
            company.siret,
            company.legal_form,
            company.naf_code,
            company.capital,
            company.employee_count,
            company.group_employee_count,
            company.director_name,
            company.phone,
            company.email,
            company.website,
            company.activity_domains,
            company.missions,
            company.archived_at,
            company.created_at,
            company.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        replace_addresses(&mut tx, company).await?;
        hydrate(&mut tx, row).await
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
            SELECT id, org_id, name, group_name, siret, legal_form, naf_code, capital, employee_count, group_employee_count, director_name, phone, email, website, activity_domains, missions, archived_at, created_at, updated_at
            FROM cursus.host_company
            WHERE org_id = $1 AND id = $2
            "#,
            organization_id.0,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        match row {
            Some(row) => Ok(Some(hydrate(&mut tx, row).await?)),
            None => Ok(None),
        }
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
            SELECT id, org_id, name, group_name, siret, legal_form, naf_code, capital, employee_count, group_employee_count, director_name, phone, email, website, activity_domains, missions, archived_at, created_at, updated_at
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

        let ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
        let addresses = load_addresses(&mut tx, organization_id, &ids).await?;

        Ok((assemble(rows, addresses), total as u64))
    }

    async fn update(&mut self, company: &HostCompany) -> Result<HostCompany, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            HostCompanyRow,
            r#"
            UPDATE cursus.host_company
            SET name = $3, group_name = $4, siret = $5, legal_form = $6, naf_code = $7,
                capital = $8, employee_count = $9, group_employee_count = $10,
                director_name = $11, phone = $12, email = $13, website = $14,
                activity_domains = $15, missions = $16, updated_at = $17
            WHERE org_id = $1 AND id = $2
            RETURNING id, org_id, name, group_name, siret, legal_form, naf_code, capital, employee_count, group_employee_count, director_name, phone, email, website, activity_domains, missions, archived_at, created_at, updated_at
            "#,
            company.organization_id.0,
            company.id.0,
            company.name,
            company.group_name,
            company.siret,
            company.legal_form,
            company.naf_code,
            company.capital,
            company.employee_count,
            company.group_employee_count,
            company.director_name,
            company.phone,
            company.email,
            company.website,
            company.activity_domains,
            company.missions,
            company.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .ok_or(CoreError::NotFound)?;

        replace_addresses(&mut tx, company).await?;
        hydrate(&mut tx, row).await
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
            RETURNING id, org_id, name, group_name, siret, legal_form, naf_code, capital, employee_count, group_employee_count, director_name, phone, email, website, activity_domains, missions, archived_at, created_at, updated_at
            "#,
            organization_id.0,
            id.0,
            archived_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .ok_or(CoreError::NotFound)?;

        hydrate(&mut tx, row).await
    }
}

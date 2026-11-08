use common::CoreError;
use mestier_macros::repository;

use crate::{
    OrganizationId, Surcharge, SurchargeId,
    domain::surcharge::ports::SurchargeRepository,
    infrastructure::{
        postgres::{SharedTx, error::map_sqlx_error},
        surcharge::postgres::model::SurchargeRow,
    },
};

#[repository(domain = Surcharge, backend = Postgres)]
pub struct PgSurchargeRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgSurchargeRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> SurchargeRepository for PgSurchargeRepository<'tx> {
    async fn lock_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            "SELECT pg_advisory_xact_lock(hashtextextended('surcharges:' || $1::text, 0))",
            organization_id.0.to_string(),
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Surcharge>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            SurchargeRow,
            r#"
            SELECT id, org_id, name, rate_bp, kind, archived_at, created_at
            FROM surcharges
            WHERE org_id = $1
            ORDER BY created_at, id
            "#,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(Surcharge::try_from).collect()
    }

    async fn find(
        &mut self,
        organization_id: OrganizationId,
        id: SurchargeId,
    ) -> Result<Option<Surcharge>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            SurchargeRow,
            r#"
            SELECT id, org_id, name, rate_bp, kind, archived_at, created_at
            FROM surcharges
            WHERE org_id = $1 AND id = $2
            "#,
            organization_id.0,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Surcharge::try_from).transpose()
    }

    async fn insert(&mut self, surcharge: &Surcharge) -> Result<Surcharge, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            SurchargeRow,
            r#"
            INSERT INTO surcharges (id, org_id, name, rate_bp, kind, archived_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, org_id, name, rate_bp, kind, archived_at, created_at
            "#,
            surcharge.id.0,
            surcharge.organization_id.0,
            surcharge.name.as_str(),
            surcharge.rate.get(),
            surcharge.kind.as_str(),
            surcharge.archived_at,
            surcharge.created_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Surcharge::try_from(row)
    }

    async fn update(&mut self, surcharge: &Surcharge) -> Result<Surcharge, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            SurchargeRow,
            r#"
            UPDATE surcharges
            SET name = $3, rate_bp = $4, kind = $5, archived_at = $6
            WHERE org_id = $1 AND id = $2
            RETURNING id, org_id, name, rate_bp, kind, archived_at, created_at
            "#,
            surcharge.organization_id.0,
            surcharge.id.0,
            surcharge.name.as_str(),
            surcharge.rate.get(),
            surcharge.kind.as_str(),
            surcharge.archived_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.ok_or(CoreError::NotFound).and_then(Surcharge::try_from)
    }
}

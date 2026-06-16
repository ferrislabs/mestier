use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    OrganizationId, ServiceRate, ServiceRateId,
    domain::service_rate::ports::ServiceRateRepository,
    infrastructure::{
        postgres::{SharedTx, error::map_sqlx_error},
        service_rate::postgres::model::ServiceRateRow,
    },
};

#[repository(domain = ServiceRate, backend = Postgres)]
pub struct PgServiceRateRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgServiceRateRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> ServiceRateRepository for PgServiceRateRepository<'tx> {
    async fn insert(&mut self, service_rate: &ServiceRate) -> Result<ServiceRate, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ServiceRateRow,
            r#"
            INSERT INTO service_rates (id, org_id, label, unit, rate_cents, deleted_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, org_id, label, unit, rate_cents, deleted_at, created_at, updated_at
            "#,
            service_rate.id.0,
            service_rate.organization_id.0,
            service_rate.label,
            service_rate.unit.as_str(),
            service_rate.rate_cents,
            service_rate.deleted_at,
            service_rate.created_at,
            service_rate.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.try_into()
    }

    async fn find_by_id(&mut self, id: ServiceRateId) -> Result<Option<ServiceRate>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ServiceRateRow,
            r#"
            SELECT id, org_id, label, unit, rate_cents, deleted_at, created_at, updated_at
            FROM service_rates
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
    ) -> Result<(Vec<ServiceRate>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            ServiceRateRow,
            r#"
            SELECT id, org_id, label, unit, rate_cents, deleted_at, created_at, updated_at
            FROM service_rates
            WHERE org_id = $1 AND deleted_at IS NULL
            ORDER BY label ASC, created_at ASC
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
            r#"SELECT COUNT(*) AS "count!" FROM service_rates WHERE org_id = $1 AND deleted_at IS NULL"#,
            organization_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let items = rows
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, CoreError>>()?;

        Ok((items, total as u64))
    }

    async fn update(&mut self, service_rate: &ServiceRate) -> Result<ServiceRate, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ServiceRateRow,
            r#"
            UPDATE service_rates
            SET label = $2, unit = $3, rate_cents = $4, updated_at = $5
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, org_id, label, unit, rate_cents, deleted_at, created_at, updated_at
            "#,
            service_rate.id.0,
            service_rate.label,
            service_rate.unit.as_str(),
            service_rate.rate_cents,
            service_rate.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(TryInto::try_into)
            .transpose()?
            .ok_or(CoreError::NotFound)
    }

    async fn soft_delete(
        &mut self,
        id: ServiceRateId,
        deleted_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE service_rates
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

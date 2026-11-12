use common::CoreError;
use mestier_macros::repository;

use crate::{
    FeatureActivation, FeatureFlag, OrganizationId, UserId,
    domain::feature_flag::ports::FeatureFlagRepository,
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

#[repository(domain = FeatureFlag, backend = Postgres)]
pub struct PgFeatureFlagRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgFeatureFlagRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> FeatureFlagRepository for PgFeatureFlagRepository<'tx> {
    async fn list_enabled(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<FeatureActivation>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query!(
            r#"
            SELECT flag, enabled_at, enabled_by
            FROM organization_feature_flags
            WHERE org_id = $1
            ORDER BY enabled_at, flag
            "#,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                Some(FeatureActivation {
                    flag: FeatureFlag::try_from(row.flag.as_str()).ok()?,
                    enabled_at: row.enabled_at,
                    enabled_by: row.enabled_by.map(UserId),
                })
            })
            .collect())
    }

    async fn is_enabled(
        &mut self,
        organization_id: OrganizationId,
        flag: FeatureFlag,
    ) -> Result<bool, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query!(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM organization_feature_flags
                WHERE org_id = $1 AND flag = $2
            ) AS "enabled!"
            "#,
            organization_id.0,
            flag.key(),
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.enabled)
    }

    async fn enable(
        &mut self,
        organization_id: OrganizationId,
        flag: FeatureFlag,
        by: UserId,
    ) -> Result<bool, CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            INSERT INTO organization_feature_flags (org_id, flag, enabled_by)
            VALUES ($1, $2, $3)
            ON CONFLICT (org_id, flag) DO NOTHING
            "#,
            organization_id.0,
            flag.key(),
            by.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(result.rows_affected() == 1)
    }

    async fn disable(
        &mut self,
        organization_id: OrganizationId,
        flag: FeatureFlag,
    ) -> Result<bool, CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            DELETE FROM organization_feature_flags
            WHERE org_id = $1 AND flag = $2
            "#,
            organization_id.0,
            flag.key(),
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(result.rows_affected() == 1)
    }
}

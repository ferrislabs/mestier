use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    Campaign, CampaignId, OrganizationId,
    domain::survey::ports::CampaignRepository,
    infrastructure::{
        postgres::{SharedTx, error::map_sqlx_error},
        survey::postgres::model::{CampaignRow, campaign_metadata},
    },
};

#[repository(domain = SurveyCampaign, backend = Postgres)]
pub struct PgCampaignRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgCampaignRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

fn campaigns(rows: Vec<CampaignRow>) -> Result<Vec<Campaign>, CoreError> {
    rows.into_iter().map(Campaign::try_from).collect()
}

impl<'tx> CampaignRepository for PgCampaignRepository<'tx> {
    async fn find(
        &mut self,
        organization_id: OrganizationId,
        id: CampaignId,
    ) -> Result<Option<Campaign>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CampaignRow,
            r#"
            SELECT id, org_id, name, starts_at, ends_at, status, metadata
            FROM survey.campaign
            WHERE org_id = $1 AND id = $2
            "#,
            organization_id.0,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Campaign::try_from).transpose()
    }

    async fn insert(&mut self, campaign: &Campaign) -> Result<Campaign, CoreError> {
        let metadata = campaign_metadata(campaign)?;
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CampaignRow,
            r#"
            INSERT INTO survey.campaign (
                id, org_id, name, starts_at, ends_at, status, metadata, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, now(), now())
            RETURNING id, org_id, name, starts_at, ends_at, status, metadata
            "#,
            campaign.id().0,
            campaign.organization_id().0,
            campaign.name(),
            campaign.window().starts_at(),
            campaign.window().ends_at(),
            campaign.status().as_str(),
            metadata,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Campaign::try_from(row)
    }

    async fn update(&mut self, campaign: &Campaign) -> Result<Campaign, CoreError> {
        let metadata = campaign_metadata(campaign)?;
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CampaignRow,
            r#"
            UPDATE survey.campaign
            SET name = $3, starts_at = $4, ends_at = $5, status = $6, metadata = $7,
                updated_at = now()
            WHERE org_id = $1 AND id = $2
            RETURNING id, org_id, name, starts_at, ends_at, status, metadata
            "#,
            campaign.organization_id().0,
            campaign.id().0,
            campaign.name(),
            campaign.window().starts_at(),
            campaign.window().ends_at(),
            campaign.status().as_str(),
            metadata,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.ok_or(CoreError::NotFound).and_then(Campaign::try_from)
    }

    async fn list(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Campaign>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let total = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM survey.campaign WHERE org_id = $1"#,
            organization_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        let rows = sqlx::query_as!(
            CampaignRow,
            r#"
            SELECT id, org_id, name, starts_at, ends_at, status, metadata
            FROM survey.campaign
            WHERE org_id = $1
            ORDER BY created_at DESC, id DESC
            LIMIT $2 OFFSET $3
            "#,
            organization_id.0,
            i64::try_from(limit).unwrap_or(i64::MAX),
            i64::try_from(offset).unwrap_or(i64::MAX),
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok((campaigns(rows)?, u64::try_from(total).unwrap_or(0)))
    }

    async fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: CampaignId,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let deleted = sqlx::query!(
            "DELETE FROM survey.campaign WHERE org_id = $1 AND id = $2",
            organization_id.0,
            id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        if deleted.rows_affected() == 0 {
            Err(CoreError::NotFound)
        } else {
            Ok(())
        }
    }

    async fn list_due(
        &mut self,
        now: DateTime<Utc>,
        limit: u64,
    ) -> Result<Vec<Campaign>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            CampaignRow,
            r#"
            SELECT id, org_id, name, starts_at, ends_at, status, metadata
            FROM survey.campaign
            WHERE (status = 'SCHEDULED' AND (starts_at IS NULL OR starts_at <= $1))
               OR (status = 'RUNNING' AND ends_at IS NOT NULL AND ends_at <= $1)
            ORDER BY COALESCE(
                CASE WHEN status = 'SCHEDULED' THEN starts_at ELSE ends_at END,
                created_at
            ), id
            LIMIT $2
            "#,
            now,
            i64::try_from(limit).unwrap_or(i64::MAX),
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        campaigns(rows)
    }

    async fn find_for_update(
        &mut self,
        organization_id: OrganizationId,
        id: CampaignId,
    ) -> Result<Option<Campaign>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CampaignRow,
            r#"
            SELECT id, org_id, name, starts_at, ends_at, status, metadata
            FROM survey.campaign
            WHERE org_id = $1 AND id = $2
            FOR UPDATE
            "#,
            organization_id.0,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Campaign::try_from).transpose()
    }
}

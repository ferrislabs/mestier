use common::CoreError;
use mestier_macros::repository;

use crate::{
    CampaignId, OrganizationId, PublicSlug, SurveyError, SurveyId, SurveyRecord,
    domain::survey::ports::SurveyRepository,
    infrastructure::{
        postgres::{SharedTx, error::map_sqlx_error},
        survey::postgres::model::{SurveyColumns, SurveyRow},
    },
};

#[repository(domain = Survey, backend = Postgres)]
pub struct PgSurveyRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgSurveyRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

fn records(rows: Vec<SurveyRow>) -> Result<Vec<SurveyRecord>, CoreError> {
    rows.into_iter().map(SurveyRecord::try_from).collect()
}

impl<'tx> SurveyRepository for PgSurveyRepository<'tx> {
    async fn find(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
    ) -> Result<Option<SurveyRecord>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            SurveyRow,
            r#"
            SELECT id, org_id, campaign_id, title, description, visibility, graph,
                   allow_multiple_responses, response_cap, metadata, status, public_slug,
                   opened_at, closed_at
            FROM survey.survey
            WHERE org_id = $1 AND id = $2
            "#,
            organization_id.0,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(SurveyRecord::try_from).transpose()
    }

    async fn insert(&mut self, survey: &SurveyRecord) -> Result<SurveyRecord, CoreError> {
        let columns = SurveyColumns::try_from(survey)?;
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            SurveyRow,
            r#"
            INSERT INTO survey.survey (
                id, org_id, campaign_id, title, description, visibility, graph,
                allow_multiple_responses, response_cap, metadata, status, public_slug,
                opened_at, closed_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, now(), now())
            RETURNING id, org_id, campaign_id, title, description, visibility, graph,
                      allow_multiple_responses, response_cap, metadata, status, public_slug,
                      opened_at, closed_at
            "#,
            columns.id,
            columns.org_id,
            columns.campaign_id,
            columns.title,
            columns.description,
            columns.visibility,
            columns.graph,
            columns.allow_multiple_responses,
            columns.response_cap,
            columns.metadata,
            columns.status,
            columns.public_slug,
            columns.opened_at,
            columns.closed_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        SurveyRecord::try_from(row)
    }

    async fn update(&mut self, survey: &SurveyRecord) -> Result<SurveyRecord, CoreError> {
        let columns = SurveyColumns::try_from(survey)?;
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            SurveyRow,
            r#"
            UPDATE survey.survey
            SET campaign_id = $3, title = $4, description = $5, visibility = $6, graph = $7,
                allow_multiple_responses = $8, response_cap = $9, metadata = $10, status = $11,
                public_slug = $12, opened_at = $13, closed_at = $14, updated_at = now()
            WHERE org_id = $1 AND id = $2
              AND CASE status WHEN 'DRAFT' THEN 0 WHEN 'OPENED' THEN 1 ELSE 2 END
                  <= CASE $11::text WHEN 'DRAFT' THEN 0 WHEN 'OPENED' THEN 1 ELSE 2 END
            RETURNING id, org_id, campaign_id, title, description, visibility, graph,
                      allow_multiple_responses, response_cap, metadata, status, public_slug,
                      opened_at, closed_at
            "#,
            columns.org_id,
            columns.id,
            columns.campaign_id,
            columns.title,
            columns.description,
            columns.visibility,
            columns.graph,
            columns.allow_multiple_responses,
            columns.response_cap,
            columns.metadata,
            columns.status,
            columns.public_slug,
            columns.opened_at,
            columns.closed_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        if let Some(row) = row {
            return SurveyRecord::try_from(row);
        }
        let exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM survey.survey WHERE org_id = $1 AND id = $2
            ) AS "exists!"
            "#,
            columns.org_id,
            columns.id,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        if exists {
            Err(SurveyError::SurveyStateChanged.into())
        } else {
            Err(CoreError::NotFound)
        }
    }

    async fn list(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<SurveyRecord>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let total = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM survey.survey WHERE org_id = $1"#,
            organization_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        let rows = sqlx::query_as!(
            SurveyRow,
            r#"
            SELECT id, org_id, campaign_id, title, description, visibility, graph,
                   allow_multiple_responses, response_cap, metadata, status, public_slug,
                   opened_at, closed_at
            FROM survey.survey
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

        Ok((records(rows)?, u64::try_from(total).unwrap_or(0)))
    }

    async fn list_by_campaign(
        &mut self,
        organization_id: OrganizationId,
        campaign_id: CampaignId,
    ) -> Result<Vec<SurveyRecord>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            SurveyRow,
            r#"
            SELECT id, org_id, campaign_id, title, description, visibility, graph,
                   allow_multiple_responses, response_cap, metadata, status, public_slug,
                   opened_at, closed_at
            FROM survey.survey
            WHERE org_id = $1 AND campaign_id = $2
            ORDER BY created_at DESC, id DESC
            "#,
            organization_id.0,
            campaign_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        records(rows)
    }

    async fn find_by_public_slug(
        &mut self,
        slug: &PublicSlug,
    ) -> Result<Option<SurveyRecord>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            SurveyRow,
            r#"
            SELECT id, org_id, campaign_id, title, description, visibility, graph,
                   allow_multiple_responses, response_cap, metadata, status, public_slug,
                   opened_at, closed_at
            FROM survey.survey
            WHERE public_slug = $1
            "#,
            slug.as_str(),
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(SurveyRecord::try_from).transpose()
    }

    async fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let deleted = sqlx::query!(
            "DELETE FROM survey.survey WHERE org_id = $1 AND id = $2",
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

    async fn has_responses(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
    ) -> Result<bool, CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM survey.response WHERE org_id = $1 AND survey_id = $2
            ) AS "exists!"
            "#,
            organization_id.0,
            id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)
    }

    async fn find_for_update(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
    ) -> Result<Option<SurveyRecord>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            SurveyRow,
            r#"
            SELECT id, org_id, campaign_id, title, description, visibility, graph,
                   allow_multiple_responses, response_cap, metadata, status, public_slug,
                   opened_at, closed_at
            FROM survey.survey
            WHERE org_id = $1 AND id = $2
            FOR UPDATE
            "#,
            organization_id.0,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(SurveyRecord::try_from).transpose()
    }
}

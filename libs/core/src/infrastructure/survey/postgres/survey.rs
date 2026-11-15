use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    CampaignId, OrganizationId, PublicSlug, SurveyError, SurveyGraph as Graph, SurveyGraphVersion,
    SurveyId, SurveyRecord, SurveyState, SurveySummary,
    domain::survey::{Opened, Survey, ports::SurveyRepository},
    infrastructure::{
        postgres::{SharedTx, error::map_sqlx_error},
        survey::postgres::model::{SurveyColumns, SurveyRow, version_column},
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

    async fn write_campaigns(
        &mut self,
        org_id: uuid::Uuid,
        id: uuid::Uuid,
        campaign_ids: &[uuid::Uuid],
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            r#"
            DELETE FROM survey.survey_campaign
            WHERE org_id = $1 AND survey_id = $2 AND NOT (campaign_id = ANY($3::uuid[]))
            "#,
            org_id,
            id,
            campaign_ids,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        sqlx::query!(
            r#"
            INSERT INTO survey.survey_campaign (org_id, survey_id, campaign_id, position)
            SELECT $1, $2, link.campaign_id, (link.ordinal - 1)::integer
            FROM UNNEST($3::uuid[]) WITH ORDINALITY AS link(campaign_id, ordinal)
            ON CONFLICT (survey_id, campaign_id) DO UPDATE SET position = EXCLUDED.position
            "#,
            org_id,
            id,
            campaign_ids,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn reloaded(&mut self, columns: &SurveyColumns<'_>) -> Result<SurveyRecord, CoreError> {
        self.find(OrganizationId(columns.org_id), SurveyId(columns.id))
            .await?
            .ok_or(CoreError::NotFound)
    }
}

fn state_of(status: &str, id: uuid::Uuid) -> Result<SurveyState, CoreError> {
    match status {
        "DRAFT" => Ok(SurveyState::Draft),
        "OPENED" => Ok(SurveyState::Opened),
        "CLOSED" => Ok(SurveyState::Closed),
        other => Err(CoreError::Internal(format!(
            "survey.survey {id}: invalid status {other}"
        ))),
    }
}

fn bound(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
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
            SELECT s.id, s.org_id,
                   COALESCE(
                       (SELECT array_agg(sc.campaign_id ORDER BY sc.position, sc.attached_at)
                        FROM survey.survey_campaign sc
                        WHERE sc.org_id = s.org_id AND sc.survey_id = s.id),
                       '{}'
                   ) AS "campaign_ids!",
                   s.title, s.description, s.visibility, s.graph, s.graph_version,
                   s.allow_multiple_responses, s.response_cap, s.metadata, s.status,
                   s.public_slug, s.opened_at, s.closed_at
            FROM survey.survey s
            WHERE s.org_id = $1 AND s.id = $2
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
        {
            let mut tx = self.tx.lock().await;
            sqlx::query!(
                r#"
                INSERT INTO survey.survey (
                    id, org_id, title, description, visibility, graph, graph_version,
                    allow_multiple_responses, response_cap, metadata, status, public_slug,
                    opened_at, closed_at, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, now(), now())
                "#,
                columns.id,
                columns.org_id,
                columns.title,
                columns.description,
                columns.visibility,
                columns.graph,
                columns.graph_version,
                columns.allow_multiple_responses,
                columns.response_cap,
                columns.metadata,
                columns.status,
                columns.public_slug,
                columns.opened_at,
                columns.closed_at,
            )
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        }
        self.write_campaigns(columns.org_id, columns.id, &columns.campaign_ids)
            .await?;
        self.reloaded(&columns).await
    }

    async fn update(&mut self, survey: &SurveyRecord) -> Result<SurveyRecord, CoreError> {
        let columns = SurveyColumns::try_from(survey)?;
        {
            let mut tx = self.tx.lock().await;
            let updated = sqlx::query!(
                r#"
                UPDATE survey.survey
                SET title = $3, description = $4, visibility = $5, graph = $6,
                    graph_version = $7, allow_multiple_responses = $8, response_cap = $9,
                    metadata = $10, status = $11, public_slug = $12, opened_at = $13,
                    closed_at = $14, updated_at = now()
                WHERE org_id = $1 AND id = $2
                  AND CASE status WHEN 'DRAFT' THEN 0 WHEN 'OPENED' THEN 1 ELSE 2 END
                      <= CASE $11::text WHEN 'DRAFT' THEN 0 WHEN 'OPENED' THEN 1 ELSE 2 END
                "#,
                columns.org_id,
                columns.id,
                columns.title,
                columns.description,
                columns.visibility,
                columns.graph,
                columns.graph_version,
                columns.allow_multiple_responses,
                columns.response_cap,
                columns.metadata,
                columns.status,
                columns.public_slug,
                columns.opened_at,
                columns.closed_at,
            )
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;

            if updated.rows_affected() == 0 {
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

                return if exists {
                    Err(SurveyError::SurveyStateChanged.into())
                } else {
                    Err(CoreError::NotFound)
                };
            }
        }
        self.reloaded(&columns).await
    }

    async fn reopen(
        &mut self,
        survey: &Survey<Opened>,
        closed_at: DateTime<Utc>,
    ) -> Result<SurveyRecord, CoreError> {
        let organization_id = survey.organization_id();
        let id = survey.id();
        {
            let mut tx = self.tx.lock().await;
            let updated = sqlx::query!(
                r#"
                UPDATE survey.survey
                SET status = 'OPENED', closed_at = NULL, updated_at = now()
                WHERE org_id = $1 AND id = $2 AND status = 'CLOSED' AND closed_at = $3
                "#,
                organization_id.0,
                id.0,
                closed_at,
            )
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;

            if updated.rows_affected() == 0 {
                let exists = sqlx::query_scalar!(
                    r#"
                    SELECT EXISTS (
                        SELECT 1 FROM survey.survey WHERE org_id = $1 AND id = $2
                    ) AS "exists!"
                    "#,
                    organization_id.0,
                    id.0,
                )
                .fetch_one(&mut ***tx)
                .await
                .map_err(map_sqlx_error)?;

                return if exists {
                    Err(SurveyError::SurveyStateChanged.into())
                } else {
                    Err(CoreError::NotFound)
                };
            }
        }
        self.find(organization_id, id)
            .await?
            .ok_or(CoreError::NotFound)
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
            SELECT s.id, s.org_id,
                   COALESCE(
                       (SELECT array_agg(sc.campaign_id ORDER BY sc.position, sc.attached_at)
                        FROM survey.survey_campaign sc
                        WHERE sc.org_id = s.org_id AND sc.survey_id = s.id),
                       '{}'
                   ) AS "campaign_ids!",
                   s.title, s.description, s.visibility, s.graph, s.graph_version,
                   s.allow_multiple_responses, s.response_cap, s.metadata, s.status,
                   s.public_slug, s.opened_at, s.closed_at
            FROM survey.survey s
            WHERE s.org_id = $1
            ORDER BY s.created_at DESC, s.id DESC
            LIMIT $2 OFFSET $3
            "#,
            organization_id.0,
            bound(limit),
            bound(offset),
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
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<SurveyRecord>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let total = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!" FROM survey.survey_campaign
            WHERE org_id = $1 AND campaign_id = $2
            "#,
            organization_id.0,
            campaign_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        let rows = sqlx::query_as!(
            SurveyRow,
            r#"
            SELECT s.id, s.org_id,
                   COALESCE(
                       (SELECT array_agg(sc.campaign_id ORDER BY sc.position, sc.attached_at)
                        FROM survey.survey_campaign sc
                        WHERE sc.org_id = s.org_id AND sc.survey_id = s.id),
                       '{}'
                   ) AS "campaign_ids!",
                   s.title, s.description, s.visibility, s.graph, s.graph_version,
                   s.allow_multiple_responses, s.response_cap, s.metadata, s.status,
                   s.public_slug, s.opened_at, s.closed_at
            FROM survey.survey s
            JOIN survey.survey_campaign link
              ON link.org_id = s.org_id AND link.survey_id = s.id AND link.campaign_id = $2
            WHERE s.org_id = $1
            ORDER BY s.created_at DESC, s.id DESC
            LIMIT $3 OFFSET $4
            "#,
            organization_id.0,
            campaign_id.0,
            bound(limit),
            bound(offset),
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok((records(rows)?, u64::try_from(total).unwrap_or(0)))
    }

    async fn list_summaries_by_campaign(
        &mut self,
        organization_id: OrganizationId,
        campaign_id: CampaignId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<SurveySummary>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let total = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!" FROM survey.survey_campaign
            WHERE org_id = $1 AND campaign_id = $2
            "#,
            organization_id.0,
            campaign_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        let rows = sqlx::query!(
            r#"
            SELECT s.id, s.title, s.status
            FROM survey.survey s
            JOIN survey.survey_campaign link
              ON link.org_id = s.org_id AND link.survey_id = s.id AND link.campaign_id = $2
            WHERE s.org_id = $1
            ORDER BY s.created_at DESC, s.id DESC
            LIMIT $3 OFFSET $4
            "#,
            organization_id.0,
            campaign_id.0,
            bound(limit),
            bound(offset),
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let summaries = rows
            .into_iter()
            .map(|row| {
                Ok(SurveySummary {
                    id: SurveyId(row.id),
                    state: state_of(&row.status, row.id)?,
                    title: row.title,
                })
            })
            .collect::<Result<Vec<_>, CoreError>>()?;
        Ok((summaries, u64::try_from(total).unwrap_or(0)))
    }

    async fn save_campaigns(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
        campaign_ids: &[CampaignId],
    ) -> Result<(), CoreError> {
        let ids: Vec<uuid::Uuid> = campaign_ids.iter().map(|campaign| campaign.0).collect();
        self.write_campaigns(organization_id.0, id.0, &ids).await
    }

    async fn find_by_public_slug(
        &mut self,
        slug: &PublicSlug,
    ) -> Result<Option<SurveyRecord>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            SurveyRow,
            r#"
            SELECT s.id, s.org_id,
                   COALESCE(
                       (SELECT array_agg(sc.campaign_id ORDER BY sc.position, sc.attached_at)
                        FROM survey.survey_campaign sc
                        WHERE sc.org_id = s.org_id AND sc.survey_id = s.id),
                       '{}'
                   ) AS "campaign_ids!",
                   s.title, s.description, s.visibility, s.graph, s.graph_version,
                   s.allow_multiple_responses, s.response_cap, s.metadata, s.status,
                   s.public_slug, s.opened_at, s.closed_at
            FROM survey.survey s
            WHERE s.public_slug = $1
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

    async fn has_graph_version(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
        version: SurveyGraphVersion,
    ) -> Result<bool, CoreError> {
        let version = version_column("survey", id.0, version)?;
        let mut tx = self.tx.lock().await;
        sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM survey.survey_version
                WHERE org_id = $1 AND survey_id = $2 AND version = $3
            ) AS "exists!"
            "#,
            organization_id.0,
            id.0,
            version,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)
    }

    async fn store_graph_version(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
        version: SurveyGraphVersion,
        graph: &Graph,
    ) -> Result<(), CoreError> {
        let version = version_column("survey", id.0, version)?;
        let graph = serde_json::to_value(graph)
            .map_err(|error| CoreError::Internal(format!("survey.survey {}: {error}", id.0)))?;
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            r#"
            INSERT INTO survey.survey_version (org_id, survey_id, version, graph)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (survey_id, version) DO NOTHING
            "#,
            organization_id.0,
            id.0,
            version,
            graph,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn find_graph_version(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
        version: SurveyGraphVersion,
    ) -> Result<Option<Graph>, CoreError> {
        let version = version_column("survey", id.0, version)?;
        let mut tx = self.tx.lock().await;
        let graph = sqlx::query_scalar!(
            r#"
            SELECT graph FROM survey.survey_version
            WHERE org_id = $1 AND survey_id = $2 AND version = $3
            "#,
            organization_id.0,
            id.0,
            version,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        graph
            .map(|value| {
                serde_json::from_value(value).map_err(|error| {
                    CoreError::Internal(format!("survey.survey_version {}: {error}", id.0))
                })
            })
            .transpose()
    }

    async fn find_for_update(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
    ) -> Result<Option<SurveyRecord>, CoreError> {
        let locked = {
            let mut tx = self.tx.lock().await;
            sqlx::query_scalar!(
                r#"SELECT 1 AS "locked!" FROM survey.survey WHERE org_id = $1 AND id = $2 FOR UPDATE"#,
                organization_id.0,
                id.0,
            )
            .fetch_optional(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
        };
        match locked {
            Some(_) => self.find(organization_id, id).await,
            None => Ok(None),
        }
    }
}

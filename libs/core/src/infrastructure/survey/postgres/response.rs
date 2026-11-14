use common::CoreError;
use mestier_macros::repository;

use crate::{
    MemberId, OrganizationId, ResponseId, ResponseRecord, ResumeToken, SurveyError, SurveyId,
    domain::survey::{Completed, Response, ports::ResponseRepository},
    infrastructure::{
        postgres::{SharedTx, error::map_sqlx_error},
        survey::postgres::model::{ResponseColumns, ResponseRow, map_response_error},
    },
};

#[repository(domain = SurveyResponse, backend = Postgres)]
pub struct PgResponseRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgResponseRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

fn records(rows: Vec<ResponseRow>) -> Result<Vec<ResponseRecord>, CoreError> {
    rows.into_iter().map(ResponseRecord::try_from).collect()
}

fn page(limit: u64, offset: u64) -> (i64, i64) {
    (
        i64::try_from(limit).unwrap_or(i64::MAX),
        i64::try_from(offset).unwrap_or(i64::MAX),
    )
}

impl<'tx> ResponseRepository for PgResponseRepository<'tx> {
    async fn find(
        &mut self,
        organization_id: OrganizationId,
        id: ResponseId,
    ) -> Result<Option<ResponseRecord>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ResponseRow,
            r#"
            SELECT id, org_id, survey_id, member_id, answers, status, resume_token,
                   identifier_value, unique_respondent, completed_at, created_at
            FROM survey.response
            WHERE org_id = $1 AND id = $2
            "#,
            organization_id.0,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(ResponseRecord::try_from).transpose()
    }

    async fn insert(&mut self, response: &ResponseRecord) -> Result<ResponseRecord, CoreError> {
        let columns = ResponseColumns::try_from(response)?;
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ResponseRow,
            r#"
            INSERT INTO survey.response (
                id, org_id, survey_id, member_id, answers, status, resume_token,
                identifier_value, unique_respondent, completed_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, now())
            RETURNING id, org_id, survey_id, member_id, answers, status, resume_token,
                      identifier_value, unique_respondent, completed_at, created_at
            "#,
            columns.id,
            columns.org_id,
            columns.survey_id,
            columns.member_id,
            columns.answers,
            columns.status,
            columns.resume_token,
            columns.identifier_value,
            columns.unique_respondent,
            columns.completed_at,
            columns.created_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_response_error)?;

        ResponseRecord::try_from(row)
    }

    async fn update(&mut self, response: &ResponseRecord) -> Result<ResponseRecord, CoreError> {
        let columns = ResponseColumns::try_from(response)?;
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ResponseRow,
            r#"
            UPDATE survey.response
            SET answers = $3, status = $4, identifier_value = $5, unique_respondent = $6,
                completed_at = $7, updated_at = now()
            WHERE org_id = $1 AND id = $2 AND status = 'IN_PROGRESS'
            RETURNING id, org_id, survey_id, member_id, answers, status, resume_token,
                      identifier_value, unique_respondent, completed_at, created_at
            "#,
            columns.org_id,
            columns.id,
            columns.answers,
            columns.status,
            columns.identifier_value,
            columns.unique_respondent,
            columns.completed_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_response_error)?;

        if let Some(row) = row {
            return ResponseRecord::try_from(row);
        }
        let exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM survey.response WHERE org_id = $1 AND id = $2
            ) AS "exists!"
            "#,
            columns.org_id,
            columns.id,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        if exists {
            Err(SurveyError::ResponseAlreadyCompleted.into())
        } else {
            Err(CoreError::NotFound)
        }
    }

    async fn list(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<ResponseRecord>, u64), CoreError> {
        let (limit, offset) = page(limit, offset);
        let mut tx = self.tx.lock().await;
        let total = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM survey.response WHERE org_id = $1"#,
            organization_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        let rows = sqlx::query_as!(
            ResponseRow,
            r#"
            SELECT id, org_id, survey_id, member_id, answers, status, resume_token,
                   identifier_value, unique_respondent, completed_at, created_at
            FROM survey.response
            WHERE org_id = $1
            ORDER BY created_at DESC, id DESC
            LIMIT $2 OFFSET $3
            "#,
            organization_id.0,
            limit,
            offset,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok((records(rows)?, u64::try_from(total).unwrap_or(0)))
    }

    async fn list_for_survey(
        &mut self,
        organization_id: OrganizationId,
        survey_id: SurveyId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<ResponseRecord>, u64), CoreError> {
        let (limit, offset) = page(limit, offset);
        let mut tx = self.tx.lock().await;
        let total = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM survey.response
            WHERE org_id = $1 AND survey_id = $2
            "#,
            organization_id.0,
            survey_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        let rows = sqlx::query_as!(
            ResponseRow,
            r#"
            SELECT id, org_id, survey_id, member_id, answers, status, resume_token,
                   identifier_value, unique_respondent, completed_at, created_at
            FROM survey.response
            WHERE org_id = $1 AND survey_id = $2
            ORDER BY created_at DESC, id DESC
            LIMIT $3 OFFSET $4
            "#,
            organization_id.0,
            survey_id.0,
            limit,
            offset,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok((records(rows)?, u64::try_from(total).unwrap_or(0)))
    }

    async fn list_completed_for_survey(
        &mut self,
        organization_id: OrganizationId,
        survey_id: SurveyId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Response<Completed>>, u64), CoreError> {
        let (limit, offset) = page(limit, offset);
        let mut tx = self.tx.lock().await;
        let total = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM survey.response
            WHERE org_id = $1 AND survey_id = $2 AND status = 'COMPLETED'
            "#,
            organization_id.0,
            survey_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        let rows = sqlx::query_as!(
            ResponseRow,
            r#"
            SELECT id, org_id, survey_id, member_id, answers, status, resume_token,
                   identifier_value, unique_respondent, completed_at, created_at
            FROM survey.response
            WHERE org_id = $1 AND survey_id = $2 AND status = 'COMPLETED'
            ORDER BY completed_at DESC, id DESC
            LIMIT $3 OFFSET $4
            "#,
            organization_id.0,
            survey_id.0,
            limit,
            offset,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let completed = records(rows)?
            .into_iter()
            .filter_map(|record| match record {
                ResponseRecord::Completed(response) => Some(response),
                ResponseRecord::InProgress(_) => None,
            })
            .collect();
        Ok((completed, u64::try_from(total).unwrap_or(0)))
    }

    async fn find_by_resume_token(
        &mut self,
        organization_id: OrganizationId,
        survey_id: SurveyId,
        token: &ResumeToken,
    ) -> Result<Option<ResponseRecord>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ResponseRow,
            r#"
            SELECT id, org_id, survey_id, member_id, answers, status, resume_token,
                   identifier_value, unique_respondent, completed_at, created_at
            FROM survey.response
            WHERE org_id = $1 AND survey_id = $2 AND resume_token = $3
            "#,
            organization_id.0,
            survey_id.0,
            token.as_str(),
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(ResponseRecord::try_from).transpose()
    }

    async fn count_completed(
        &mut self,
        organization_id: OrganizationId,
        survey_id: SurveyId,
    ) -> Result<u64, CoreError> {
        let mut tx = self.tx.lock().await;
        let total = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM survey.response
            WHERE org_id = $1 AND survey_id = $2 AND status = 'COMPLETED'
            "#,
            organization_id.0,
            survey_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(u64::try_from(total).unwrap_or(0))
    }

    async fn find_completed_by_member(
        &mut self,
        organization_id: OrganizationId,
        survey_id: SurveyId,
        member_id: MemberId,
    ) -> Result<Option<ResponseRecord>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ResponseRow,
            r#"
            SELECT id, org_id, survey_id, member_id, answers, status, resume_token,
                   identifier_value, unique_respondent, completed_at, created_at
            FROM survey.response
            WHERE org_id = $1 AND survey_id = $2 AND member_id = $3 AND status = 'COMPLETED'
            ORDER BY completed_at DESC, id DESC
            LIMIT 1
            "#,
            organization_id.0,
            survey_id.0,
            member_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(ResponseRecord::try_from).transpose()
    }

    async fn find_completed_by_identifier(
        &mut self,
        organization_id: OrganizationId,
        survey_id: SurveyId,
        value: &str,
    ) -> Result<Option<ResponseRecord>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ResponseRow,
            r#"
            SELECT id, org_id, survey_id, member_id, answers, status, resume_token,
                   identifier_value, unique_respondent, completed_at, created_at
            FROM survey.response
            WHERE org_id = $1 AND survey_id = $2 AND identifier_value = $3
              AND status = 'COMPLETED'
            ORDER BY completed_at DESC, id DESC
            LIMIT 1
            "#,
            organization_id.0,
            survey_id.0,
            value,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(ResponseRecord::try_from).transpose()
    }

    async fn find_in_progress_by_member(
        &mut self,
        organization_id: OrganizationId,
        survey_id: SurveyId,
        member_id: MemberId,
    ) -> Result<Option<ResponseRecord>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ResponseRow,
            r#"
            SELECT id, org_id, survey_id, member_id, answers, status, resume_token,
                   identifier_value, unique_respondent, completed_at, created_at
            FROM survey.response
            WHERE org_id = $1 AND survey_id = $2 AND member_id = $3 AND status = 'IN_PROGRESS'
            ORDER BY created_at DESC, id DESC
            LIMIT 1
            "#,
            organization_id.0,
            survey_id.0,
            member_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(ResponseRecord::try_from).transpose()
    }

    async fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: ResponseId,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let deleted = sqlx::query!(
            "DELETE FROM survey.response WHERE org_id = $1 AND id = $2",
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
}

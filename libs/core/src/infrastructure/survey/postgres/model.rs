use std::num::NonZeroU32;

use chrono::{DateTime, Utc};
use common::CoreError;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{Campaign, PublicSlug, ResponseRecord, SurveyError, SurveyGraphVersion, SurveyRecord};

pub const UNIQUE_RESPONDENT_CONSTRAINTS: [&str; 2] =
    ["uq_response_unique_member", "uq_response_unique_identifier"];

fn corrupt(table: &str, id: Uuid, what: impl std::fmt::Display) -> CoreError {
    CoreError::Internal(format!("survey.{table} {id}: invalid {what}"))
}

fn encode<T: serde::Serialize>(table: &str, id: Uuid, value: &T) -> Result<Value, CoreError> {
    serde_json::to_value(value).map_err(|error| corrupt(table, id, error))
}

pub fn map_response_error(error: sqlx::Error) -> CoreError {
    match &error {
        sqlx::Error::Database(db_err)
            if db_err.is_unique_violation()
                && db_err
                    .constraint()
                    .is_some_and(|name| UNIQUE_RESPONDENT_CONSTRAINTS.contains(&name)) =>
        {
            SurveyError::RespondentAlreadyAnswered.into()
        }
        _ => crate::infrastructure::postgres::error::map_sqlx_error(error),
    }
}

#[derive(Debug, Clone)]
pub struct SurveyRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub campaign_ids: Vec<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub visibility: String,
    pub graph: Option<Value>,
    pub graph_version: Option<i32>,
    pub allow_multiple_responses: bool,
    pub response_cap: Option<i32>,
    pub metadata: Value,
    pub status: String,
    pub public_slug: Option<String>,
    pub opened_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
}

impl TryFrom<SurveyRow> for SurveyRecord {
    type Error = CoreError;

    fn try_from(row: SurveyRow) -> Result<Self, Self::Error> {
        let id = row.id;
        let response_cap = row
            .response_cap
            .map(|cap| {
                u32::try_from(cap)
                    .ok()
                    .and_then(|cap| NonZeroU32::try_from(cap).ok())
                    .ok_or_else(|| corrupt("survey", id, "response_cap"))
            })
            .transpose()?;
        let value = json!({
            "id": row.id,
            "organization_id": row.org_id,
            "title": row.title,
            "description": row.description,
            "visibility": row.visibility,
            "allow_multiple_responses": row.allow_multiple_responses,
            "response_cap": response_cap,
            "metadata": row.metadata,
            "campaign_ids": row.campaign_ids,
            "public_slug": row.public_slug,
            "opened_at": row.opened_at,
            "closed_at": row.closed_at,
            "graph": row.graph,
            "graph_version": row.graph_version,
        });
        let record = match row.status.as_str() {
            "DRAFT" => serde_json::from_value(value).map(Self::Draft),
            "OPENED" => serde_json::from_value(value).map(Self::Opened),
            "CLOSED" => serde_json::from_value(value).map(Self::Closed),
            other => return Err(corrupt("survey", id, format!("status {other}"))),
        };
        record.map_err(|error| corrupt("survey", id, error))
    }
}

pub struct SurveyColumns<'a> {
    pub id: Uuid,
    pub org_id: Uuid,
    pub campaign_ids: Vec<Uuid>,
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub visibility: &'static str,
    pub graph: Option<Value>,
    pub graph_version: Option<i32>,
    pub allow_multiple_responses: bool,
    pub response_cap: Option<i32>,
    pub metadata: Value,
    pub status: &'static str,
    pub public_slug: Option<&'a str>,
    pub opened_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
}

fn cap_column(id: Uuid, cap: Option<NonZeroU32>) -> Result<Option<i32>, CoreError> {
    cap.map(|cap| i32::try_from(cap.get()).map_err(|_| corrupt("survey", id, "response_cap")))
        .transpose()
}

impl<'a> TryFrom<&'a SurveyRecord> for SurveyColumns<'a> {
    type Error = CoreError;

    fn try_from(record: &'a SurveyRecord) -> Result<Self, Self::Error> {
        macro_rules! columns {
            ($survey:expr, $status:literal, $slug:expr) => {{
                let id = $survey.id().0;
                Self {
                    id,
                    org_id: $survey.organization_id().0,
                    campaign_ids: $survey
                        .campaign_ids()
                        .iter()
                        .map(|campaign| campaign.0)
                        .collect(),
                    title: $survey.title(),
                    description: $survey.description(),
                    visibility: $survey.visibility().as_str(),
                    graph: $survey
                        .graph()
                        .map(|graph| encode("survey", id, graph))
                        .transpose()?,
                    graph_version: $survey
                        .graph_version()
                        .map(|version| version_column("survey", id, version))
                        .transpose()?,
                    allow_multiple_responses: $survey.allow_multiple_responses(),
                    response_cap: cap_column(id, $survey.response_cap())?,
                    metadata: encode("survey", id, $survey.metadata())?,
                    status: $status,
                    public_slug: $slug.map(PublicSlug::as_str),
                    opened_at: $survey.opened_at(),
                    closed_at: $survey.closed_at(),
                }
            }};
        }
        Ok(match record {
            SurveyRecord::Draft(survey) => columns!(survey, "DRAFT", None::<&PublicSlug>),
            SurveyRecord::Opened(survey) => columns!(survey, "OPENED", survey.public_slug()),
            SurveyRecord::Closed(survey) => columns!(survey, "CLOSED", survey.public_slug()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct CampaignRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub status: String,
    pub metadata: Value,
}

impl TryFrom<CampaignRow> for Campaign {
    type Error = CoreError;

    fn try_from(row: CampaignRow) -> Result<Self, Self::Error> {
        let id = row.id;
        serde_json::from_value(json!({
            "id": row.id,
            "organization_id": row.org_id,
            "name": row.name,
            "window": {
                "starts_at": row.starts_at,
                "ends_at": row.ends_at,
            },
            "metadata": row.metadata,
            "status": row.status,
        }))
        .map_err(|error| corrupt("campaign", id, error))
    }
}

pub fn campaign_metadata(campaign: &Campaign) -> Result<Value, CoreError> {
    encode("campaign", campaign.id().0, campaign.metadata())
}

#[derive(Debug, Clone)]
pub struct ResponseRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub survey_id: Uuid,
    pub member_id: Option<Uuid>,
    pub answers: Value,
    pub status: String,
    pub resume_token: Option<String>,
    pub identifier_value: Option<String>,
    pub unique_respondent: bool,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub graph_version: Option<i32>,
}

impl TryFrom<ResponseRow> for ResponseRecord {
    type Error = CoreError;

    fn try_from(row: ResponseRow) -> Result<Self, Self::Error> {
        let id = row.id;
        let value = json!({
            "id": row.id,
            "survey_id": row.survey_id,
            "organization_id": row.org_id,
            "member_id": row.member_id,
            "resume_token": row.resume_token,
            "answers": row.answers,
            "started_at": row.created_at,
            "completed_at": row.completed_at,
            "identifier_value": row.identifier_value,
            "unique_respondent": row.unique_respondent,
            "graph_version": row.graph_version,
        });
        let record = match row.status.as_str() {
            "IN_PROGRESS" => serde_json::from_value(value).map(Self::InProgress),
            "COMPLETED" => serde_json::from_value(value).map(Self::Completed),
            other => return Err(corrupt("response", id, format!("status {other}"))),
        };
        record.map_err(|error| corrupt("response", id, error))
    }
}

pub struct ResponseColumns<'a> {
    pub id: Uuid,
    pub org_id: Uuid,
    pub survey_id: Uuid,
    pub member_id: Option<Uuid>,
    pub answers: Value,
    pub status: &'static str,
    pub resume_token: Option<&'a str>,
    pub identifier_value: Option<&'a str>,
    pub unique_respondent: bool,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub graph_version: Option<i32>,
}

pub fn version_column(
    table: &str,
    id: Uuid,
    version: SurveyGraphVersion,
) -> Result<i32, CoreError> {
    i32::try_from(version.get()).map_err(|_| corrupt(table, id, "graph_version"))
}

impl<'a> TryFrom<&'a ResponseRecord> for ResponseColumns<'a> {
    type Error = CoreError;

    fn try_from(record: &'a ResponseRecord) -> Result<Self, Self::Error> {
        let (response_id, status, identifier_value, unique_respondent) = match record {
            ResponseRecord::InProgress(response) => (response.id(), "IN_PROGRESS", None, false),
            ResponseRecord::Completed(response) => (
                response.id(),
                "COMPLETED",
                response.identifier_value(),
                response.unique_respondent(),
            ),
        };
        let id = response_id.0;
        let graph_version = record
            .graph_version()
            .map(|version| version_column("response", id, version))
            .transpose()?;
        macro_rules! columns {
            ($response:expr) => {
                Self {
                    id,
                    org_id: $response.organization_id().0,
                    survey_id: $response.survey_id().0,
                    member_id: $response.member_id().map(|member| member.0),
                    answers: encode("response", id, $response.answers())?,
                    status,
                    resume_token: $response.resume_token().map(|token| token.as_str()),
                    identifier_value,
                    unique_respondent,
                    completed_at: $response.completed_at(),
                    created_at: $response.started_at(),
                    graph_version,
                }
            };
        }
        Ok(match record {
            ResponseRecord::InProgress(response) => columns!(response),
            ResponseRecord::Completed(response) => columns!(response),
        })
    }
}

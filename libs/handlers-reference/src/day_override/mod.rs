use auth::Identity;
use axum::{
    Extension, Json, Router,
    extract::{Query, State},
    response::{IntoResponse, Response as AxumResponse},
};
use axum_extra::routing::{RouterExt, TypedPath};
use chrono::NaiveDate;
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use http::StatusCode;
use mestier_core::{
    DayOverride, DayOverrideError, DayOverrideId, DayOverrideInput, DeleteDayOverrideCommand,
    ListDayOverridesCommand, LunchInput, MemberId, OrganizationId, OverrideKind, PlanningPeriod,
    PutDayOverrideCommand,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::{IntoParams, ToSchema};

use crate::EmptyResponse;

pub const MAX_LIST_DAYS: i64 = 366;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list_day_overrides)
        .typed_put(put_day_override)
        .typed_delete(delete_day_override)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/day-overrides")]
pub struct DayOverridesPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/members/{member_id}/day-overrides/{date}")]
pub struct MemberDayOverridePath {
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    pub date: NaiveDate,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DayOverridesQuery {
    #[param(value_type = String, format = Date)]
    pub from: NaiveDate,
    #[param(value_type = String, format = Date)]
    pub to: NaiveDate,
    pub member_id: Option<MemberId>,
}

/// A day that replaces the contract for one member. `WORK` needs `start` and
/// `end` (`HH:MM`, end after start) and may carry a `lunch` break inside the
/// day. `OFF` carries no hours.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PutDayOverrideRequest {
    pub kind: OverrideKind,
    #[serde(default)]
    pub start: Option<String>,
    #[serde(default)]
    pub end: Option<String>,
    #[serde(default)]
    pub lunch: Option<LunchInput>,
    #[serde(default)]
    pub note: String,
}

impl From<PutDayOverrideRequest> for DayOverrideInput {
    fn from(value: PutDayOverrideRequest) -> Self {
        Self {
            kind: value.kind,
            start: value.start,
            end: value.end,
            lunch: value.lunch,
            note: value.note,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct DayOverrideResponse {
    pub id: DayOverrideId,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    #[schema(value_type = String, format = Date)]
    pub date: NaiveDate,
    pub kind: OverrideKind,
    pub start: Option<String>,
    pub end: Option<String>,
    pub lunch: Option<LunchInput>,
    pub note: String,
}

impl From<DayOverride> for DayOverrideResponse {
    fn from(value: DayOverride) -> Self {
        let work = value.shape.work();
        Self {
            id: value.id,
            organization_id: value.organization_id,
            member_id: value.member_id,
            date: value.date,
            kind: value.shape.kind(),
            start: work.map(|day| day.start().to_string()),
            end: work.map(|day| day.end().to_string()),
            lunch: value.shape.lunch().map(|lunch| LunchInput {
                start: lunch.start().to_string(),
                end: lunch.end().to_string(),
            }),
            note: value.note,
        }
    }
}

#[derive(Debug)]
pub enum DayOverrideFailure {
    Api(ApiError),
    Refused(DayOverrideError),
}

impl From<ApiError> for DayOverrideFailure {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<DayOverrideError> for DayOverrideFailure {
    fn from(error: DayOverrideError) -> Self {
        Self::Refused(error)
    }
}

impl IntoResponse for DayOverrideFailure {
    fn into_response(self) -> AxumResponse {
        match self {
            Self::Api(error) => error.into_response(),
            Self::Refused(error) => {
                let status = StatusCode::UNPROCESSABLE_ENTITY;
                (
                    status,
                    Json(json!({
                        "code": "E_UNPROCESSABLE_ENTITY",
                        "message": error.to_string(),
                        "status": status.as_u16(),
                        "details": { "field": error.field() },
                    })),
                )
                    .into_response()
            }
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/day-overrides",
    operation_id = "listDayOverrides",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        DayOverridesQuery,
    ),
    responses(
        (status = 200, description = "The overrides of the period (dates inclusive, at most 366 days), optionally of one member", body = inline(DataEnvelope<Vec<DayOverrideResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Needs planning management"),
        (status = 422, description = "The period is inverted or too long"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_day_overrides(
    path: DayOverridesPath,
    Query(query): Query<DayOverridesQuery>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<DayOverrideResponse>>, DayOverrideFailure> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let period =
        PlanningPeriod::new(query.from, query.to, MAX_LIST_DAYS).map_err(DayOverrideError::from)?;
    let overrides = state
        .usecase
        .acting_as(user_id)
        .list_day_overrides(ListDayOverridesCommand {
            actor,
            organization_id: path.organization_id,
            period,
            member_id: query.member_id,
        })
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(
        overrides.into_iter().map(Into::into).collect(),
    ))
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/members/{member_id}/day-overrides/{date}",
    operation_id = "putDayOverride",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("member_id" = MemberId, Path, description = "Member identifier"),
        ("date" = String, Path, description = "Day, YYYY-MM-DD", format = Date),
    ),
    request_body = PutDayOverrideRequest,
    responses(
        (status = 200, description = "The override of the day, a second call replaces the first", body = inline(DataEnvelope<DayOverrideResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Member not found in the organization"),
        (status = 422, description = "The override is refused, the field is named in details"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn put_day_override(
    path: MemberDayOverridePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<PutDayOverrideRequest>,
) -> Result<Response<DayOverrideResponse>, DayOverrideFailure> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let stored = state
        .usecase
        .acting_as(user_id)
        .put_day_override(PutDayOverrideCommand {
            actor,
            organization_id: path.organization_id,
            member_id: path.member_id,
            date: path.date,
            input: payload.into(),
        })
        .await
        .map_err(ApiError::from)??;

    Ok(Response::OK(stored.into()))
}

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/members/{member_id}/day-overrides/{date}",
    operation_id = "deleteDayOverride",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("member_id" = MemberId, Path, description = "Member identifier"),
        ("date" = String, Path, description = "Day, YYYY-MM-DD", format = Date),
    ),
    responses(
        (status = 204, description = "Override deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "No override for this member and day"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_day_override(
    path: MemberDayOverridePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    state
        .usecase
        .acting_as(user_id)
        .delete_day_override(DeleteDayOverrideCommand {
            actor,
            organization_id: path.organization_id,
            member_id: path.member_id,
            date: path.date,
        })
        .await?;

    Ok(Response::NoContent)
}

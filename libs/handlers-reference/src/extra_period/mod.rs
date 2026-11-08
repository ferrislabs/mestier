use auth::Identity;
use axum::{
    Extension, Json, Router,
    extract::{Query, State},
    response::{IntoResponse, Response as AxumResponse},
};
use axum_extra::routing::{RouterExt, TypedPath};
use chrono::{DateTime, NaiveDate, Utc};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
    resolve_actor,
};
use http::StatusCode;
use mestier_core::{
    CreateExtraPeriodCommand, DecideExtraPeriodCommand, ExtraPeriod, ExtraPeriodDraft,
    ExtraPeriodError, ExtraPeriodFilter, ExtraPeriodId, ExtraStatus, ListExtraPeriodsCommand,
    LunchInput, MemberId, OrganizationId, RejectExtraPeriodCommand, SurchargeId,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::{IntoParams, ToSchema};

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list_extra_periods)
        .typed_post(create_extra_period)
        .typed_post(approve_extra_period)
        .typed_post(reject_extra_period)
        .typed_post(cancel_extra_period)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/extra-periods")]
pub struct ExtraPeriodsPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/extra-periods/{period_id}/approve")]
pub struct ApproveExtraPeriodPath {
    pub organization_id: OrganizationId,
    pub period_id: ExtraPeriodId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/extra-periods/{period_id}/reject")]
pub struct RejectExtraPeriodPath {
    pub organization_id: OrganizationId,
    pub period_id: ExtraPeriodId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/extra-periods/{period_id}/cancel")]
pub struct CancelExtraPeriodPath {
    pub organization_id: OrganizationId,
    pub period_id: ExtraPeriodId,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ExtraPeriodsQuery {
    pub member_id: Option<MemberId>,
    pub status: Option<ExtraStatus>,
    #[param(value_type = Option<String>, format = Date)]
    pub from: Option<NaiveDate>,
    #[param(value_type = Option<String>, format = Date)]
    pub to: Option<NaiveDate>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[schema(
    description = "Hours worked outside the schedule. With a surcharge_id the rate is copied from that surcharge and rate_bp is ignored; without one the free rate_bp is kept (0 when absent). approve records the period as approved by the caller at once; it is ignored for a caller without planning management, whose own declaration is always pending."
)]
pub struct CreateExtraPeriodRequest {
    pub member_id: MemberId,
    #[schema(value_type = String, format = Date)]
    pub date: NaiveDate,
    pub start: String,
    pub end: String,
    #[serde(default)]
    pub lunch: Option<LunchInput>,
    pub reason: String,
    #[serde(default)]
    pub surcharge_id: Option<SurchargeId>,
    #[serde(default)]
    pub rate_bp: Option<i32>,
    #[serde(default)]
    pub approve: bool,
}

impl From<CreateExtraPeriodRequest> for ExtraPeriodDraft {
    fn from(value: CreateExtraPeriodRequest) -> Self {
        Self {
            member_id: value.member_id,
            date: value.date,
            start: value.start,
            end: value.end,
            lunch: value.lunch,
            reason: value.reason,
            surcharge_id: value.surcharge_id,
            rate_bp: value.rate_bp,
            approve: value.approve,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RejectExtraPeriodRequest {
    pub comment: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ExtraPeriodResponse {
    pub id: ExtraPeriodId,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    #[schema(value_type = String, format = Date)]
    pub date: NaiveDate,
    pub start: String,
    pub end: String,
    pub lunch: Option<LunchInput>,
    pub reason: String,
    pub surcharge_id: Option<SurchargeId>,
    pub rate_bp: i32,
    pub status: ExtraStatus,
    pub created_by_member_id: Option<MemberId>,
    pub decided_by_member_id: Option<MemberId>,
    pub decided_at: Option<DateTime<Utc>>,
    pub decision_comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<ExtraPeriod> for ExtraPeriodResponse {
    fn from(value: ExtraPeriod) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            member_id: value.member_id,
            date: value.date,
            start: value.hours.start().to_string(),
            end: value.hours.end().to_string(),
            lunch: value.hours.lunch().map(|lunch| LunchInput {
                start: lunch.start().to_string(),
                end: lunch.end().to_string(),
            }),
            reason: value.reason.as_str().to_owned(),
            surcharge_id: value.surcharge_id,
            rate_bp: value.rate.get(),
            status: value.status,
            created_by_member_id: value.created_by_member_id,
            decided_by_member_id: value.decided_by_member_id,
            decided_at: value.decided_at,
            decision_comment: value.decision_comment,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug)]
pub enum ExtraPeriodFailure {
    Api(ApiError),
    Refused(ExtraPeriodError),
}

impl From<ApiError> for ExtraPeriodFailure {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<ExtraPeriodError> for ExtraPeriodFailure {
    fn from(error: ExtraPeriodError) -> Self {
        Self::Refused(error)
    }
}

impl IntoResponse for ExtraPeriodFailure {
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
    path = "/api/v1/organizations/{organization_id}/extra-periods",
    operation_id = "listExtraPeriods",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ExtraPeriodsQuery,
        PaginationParams,
    ),
    responses(
        (status = 200, description = "Paginated extra periods, latest date first, optionally of one member, one status or a date window; a caller without planning management only gets their own", body = inline(DataEnvelope<Vec<ExtraPeriodResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Another member's periods need planning management"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_extra_periods(
    path: ExtraPeriodsPath,
    Query(query): Query<ExtraPeriodsQuery>,
    Query(pagination): Query<PaginationParams>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ExtraPeriodResponse>, ApiError> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let per_page = pagination.per_page();
    let page = pagination.page();
    let (periods, total) = state
        .usecase
        .acting_as(user_id)
        .list_extra_periods(ListExtraPeriodsCommand {
            actor,
            organization_id: path.organization_id,
            filter: ExtraPeriodFilter {
                member_id: query.member_id,
                status: query.status,
                from: query.from,
                to: query.to,
            },
            limit: per_page,
            offset: pagination.offset(),
        })
        .await?;
    let items: Vec<ExtraPeriodResponse> = periods.into_iter().map(Into::into).collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/extra-periods",
    operation_id = "createExtraPeriod",
    tag = super::TAG,
    params(("organization_id" = OrganizationId, Path, description = "Organization identifier")),
    request_body = CreateExtraPeriodRequest,
    responses(
        (status = 201, description = "Period declared, pending unless approved at once", body = inline(DataEnvelope<ExtraPeriodResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Declaring for another member needs planning management"),
        (status = 422, description = "The period is refused, the field is named in details"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_extra_period(
    path: ExtraPeriodsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateExtraPeriodRequest>,
) -> Result<Response<ExtraPeriodResponse>, ExtraPeriodFailure> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let created = state
        .usecase
        .acting_as(user_id)
        .create_extra_period(CreateExtraPeriodCommand {
            actor,
            organization_id: path.organization_id,
            draft: payload.into(),
        })
        .await
        .map_err(ApiError::from)??;

    Ok(Response::Created(created.into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/extra-periods/{period_id}/approve",
    operation_id = "approveExtraPeriod",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("period_id" = ExtraPeriodId, Path, description = "Extra period identifier"),
    ),
    responses(
        (status = 200, description = "Period approved", body = inline(DataEnvelope<ExtraPeriodResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Needs planning management"),
        (status = 404, description = "Period not found in the organization"),
        (status = 422, description = "Already decided"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn approve_extra_period(
    path: ApproveExtraPeriodPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ExtraPeriodResponse>, ExtraPeriodFailure> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let period = state
        .usecase
        .acting_as(user_id)
        .approve_extra_period(DecideExtraPeriodCommand {
            actor,
            organization_id: path.organization_id,
            id: path.period_id,
        })
        .await
        .map_err(ApiError::from)??;

    Ok(Response::OK(period.into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/extra-periods/{period_id}/reject",
    operation_id = "rejectExtraPeriod",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("period_id" = ExtraPeriodId, Path, description = "Extra period identifier"),
    ),
    request_body = RejectExtraPeriodRequest,
    responses(
        (status = 200, description = "Period rejected", body = inline(DataEnvelope<ExtraPeriodResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Needs planning management"),
        (status = 404, description = "Period not found in the organization"),
        (status = 422, description = "Blank comment, or already decided"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn reject_extra_period(
    path: RejectExtraPeriodPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<RejectExtraPeriodRequest>,
) -> Result<Response<ExtraPeriodResponse>, ExtraPeriodFailure> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let period = state
        .usecase
        .acting_as(user_id)
        .reject_extra_period(RejectExtraPeriodCommand {
            actor,
            organization_id: path.organization_id,
            id: path.period_id,
            comment: payload.comment,
        })
        .await
        .map_err(ApiError::from)??;

    Ok(Response::OK(period.into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/extra-periods/{period_id}/cancel",
    operation_id = "cancelExtraPeriod",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("period_id" = ExtraPeriodId, Path, description = "Extra period identifier"),
    ),
    responses(
        (status = 200, description = "Period cancelled", body = inline(DataEnvelope<ExtraPeriodResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Neither the creator nor a planning manager"),
        (status = 404, description = "Period not found in the organization"),
        (status = 422, description = "Already rejected or cancelled"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn cancel_extra_period(
    path: CancelExtraPeriodPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ExtraPeriodResponse>, ExtraPeriodFailure> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let period = state
        .usecase
        .acting_as(user_id)
        .cancel_extra_period(DecideExtraPeriodCommand {
            actor,
            organization_id: path.organization_id,
            id: path.period_id,
        })
        .await
        .map_err(ApiError::from)??;

    Ok(Response::OK(period.into()))
}

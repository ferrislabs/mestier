use auth::Identity;
use axum::{
    Extension, Json, Router,
    extract::{Query, State},
    response::{IntoResponse, Response as AxumResponse},
};
use axum_extra::routing::{RouterExt, TypedPath};
use chrono::{DateTime, NaiveDate, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use http::StatusCode;
use mestier_core::{
    AdjustmentKind, CreateLeaveAdjustmentCommand, DeleteLeaveAdjustmentCommand, LeaveAccountId,
    LeaveAdjustment, LeaveAdjustmentDraft, LeaveAdjustmentError, LeaveAdjustmentId,
    ListLeaveAdjustmentsCommand, MemberId, OrganizationId,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::{IntoParams, ToSchema};

use crate::EmptyResponse;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list_adjustments)
        .typed_post(create_adjustment)
        .typed_delete(delete_adjustment)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/leave-adjustments")]
pub struct LeaveAdjustmentsPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/leave-adjustments/{adjustment_id}")]
pub struct LeaveAdjustmentPath {
    pub organization_id: OrganizationId,
    pub adjustment_id: LeaveAdjustmentId,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct LeaveAdjustmentFilter {
    pub member_id: Option<MemberId>,
    pub account_id: Option<LeaveAccountId>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateLeaveAdjustmentRequest {
    pub member_id: MemberId,
    pub account_id: LeaveAccountId,
    #[schema(value_type = String, format = Date)]
    pub date: NaiveDate,
    pub days: f64,
    pub kind: AdjustmentKind,
    pub reason: String,
}

impl From<CreateLeaveAdjustmentRequest> for LeaveAdjustmentDraft {
    fn from(value: CreateLeaveAdjustmentRequest) -> Self {
        Self {
            member_id: value.member_id,
            account_id: value.account_id,
            date: value.date,
            days: value.days,
            kind: value.kind,
            reason: value.reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct LeaveAdjustmentResponse {
    pub id: LeaveAdjustmentId,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    pub account_id: LeaveAccountId,
    #[schema(value_type = String, format = Date)]
    pub date: NaiveDate,
    pub days: f64,
    pub kind: AdjustmentKind,
    pub reason: String,
    pub author_member_id: Option<MemberId>,
    pub created_at: DateTime<Utc>,
}

impl From<LeaveAdjustment> for LeaveAdjustmentResponse {
    fn from(value: LeaveAdjustment) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            member_id: value.member_id,
            account_id: value.account_id,
            date: value.date,
            days: value.days.days().as_decimal(),
            kind: value.kind,
            reason: value.reason.as_str().to_owned(),
            author_member_id: value.author_member_id,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug)]
pub enum LeaveAdjustmentFailure {
    Api(ApiError),
    Refused(LeaveAdjustmentError),
}

impl From<ApiError> for LeaveAdjustmentFailure {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<LeaveAdjustmentError> for LeaveAdjustmentFailure {
    fn from(error: LeaveAdjustmentError) -> Self {
        Self::Refused(error)
    }
}

impl IntoResponse for LeaveAdjustmentFailure {
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
    path = "/api/v1/organizations/{organization_id}/leave-adjustments",
    operation_id = "listLeaveAdjustments",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        LeaveAdjustmentFilter,
    ),
    responses(
        (status = 200, description = "Every adjustment of the organization, oldest first, optionally of one member or one account", body = inline(DataEnvelope<Vec<LeaveAdjustmentResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Balances are personal: needs planning or reference management"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_adjustments(
    path: LeaveAdjustmentsPath,
    Query(filter): Query<LeaveAdjustmentFilter>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<LeaveAdjustmentResponse>>, ApiError> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let adjustments = state
        .usecase
        .acting_as(user_id)
        .list_leave_adjustments(ListLeaveAdjustmentsCommand {
            actor,
            organization_id: path.organization_id,
            member_id: filter.member_id,
            account_id: filter.account_id,
        })
        .await?;

    Ok(Response::OK(
        adjustments.into_iter().map(Into::into).collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/leave-adjustments",
    operation_id = "createLeaveAdjustment",
    tag = super::TAG,
    params(("organization_id" = OrganizationId, Path, description = "Organization identifier")),
    request_body = CreateLeaveAdjustmentRequest,
    responses(
        (status = 201, description = "Adjustment recorded, authored by the caller's member", body = inline(DataEnvelope<LeaveAdjustmentResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 422, description = "The adjustment is refused, the field is named in details"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_adjustment(
    path: LeaveAdjustmentsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateLeaveAdjustmentRequest>,
) -> Result<Response<LeaveAdjustmentResponse>, LeaveAdjustmentFailure> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let created = state
        .usecase
        .acting_as(user_id)
        .create_leave_adjustment(CreateLeaveAdjustmentCommand {
            actor,
            organization_id: path.organization_id,
            draft: payload.into(),
        })
        .await
        .map_err(ApiError::from)??;

    Ok(Response::Created(created.into()))
}

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/leave-adjustments/{adjustment_id}",
    operation_id = "deleteLeaveAdjustment",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("adjustment_id" = LeaveAdjustmentId, Path, description = "Adjustment identifier"),
    ),
    responses(
        (status = 204, description = "Adjustment deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Adjustment not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_adjustment(
    path: LeaveAdjustmentPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    state
        .usecase
        .acting_as(user_id)
        .delete_leave_adjustment(DeleteLeaveAdjustmentCommand {
            actor,
            organization_id: path.organization_id,
            id: path.adjustment_id,
        })
        .await?;

    Ok(Response::NoContent)
}

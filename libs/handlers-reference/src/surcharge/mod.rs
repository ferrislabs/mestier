use auth::Identity;
use axum::{
    Extension, Json, Router,
    extract::State,
    response::{IntoResponse, Response as AxumResponse},
};
use axum_extra::routing::{RouterExt, TypedPath};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use http::StatusCode;
use mestier_core::{
    ArchiveSurchargeCommand, CreateSurchargeCommand, ListSurchargesCommand, OrganizationId,
    Surcharge, SurchargeDraft, SurchargeError, SurchargeId, SurchargeKind, UpdateSurchargeCommand,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

use crate::require_org_membership;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list_surcharges)
        .typed_post(create_surcharge)
        .typed_put(update_surcharge)
        .typed_post(archive_surcharge)
        .typed_post(restore_surcharge)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/surcharges")]
pub struct SurchargesPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/surcharges/{surcharge_id}")]
pub struct SurchargePath {
    pub organization_id: OrganizationId,
    pub surcharge_id: SurchargeId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/surcharges/{surcharge_id}/archive")]
pub struct ArchiveSurchargePath {
    pub organization_id: OrganizationId,
    pub surcharge_id: SurchargeId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/surcharges/{surcharge_id}/restore")]
pub struct RestoreSurchargePath {
    pub organization_id: OrganizationId,
    pub surcharge_id: SurchargeId,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SurchargeRequest {
    pub name: String,
    pub rate_bp: i32,
    pub kind: SurchargeKind,
}

impl From<SurchargeRequest> for SurchargeDraft {
    fn from(value: SurchargeRequest) -> Self {
        Self {
            name: value.name,
            rate_bp: value.rate_bp,
            kind: value.kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct SurchargeResponse {
    pub id: SurchargeId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub rate_bp: i32,
    pub kind: SurchargeKind,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<Surcharge> for SurchargeResponse {
    fn from(value: Surcharge) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            name: value.name.as_str().to_owned(),
            rate_bp: value.rate.get(),
            kind: value.kind,
            archived_at: value.archived_at,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug)]
pub enum SurchargeFailure {
    Api(ApiError),
    Refused(SurchargeError),
}

impl From<ApiError> for SurchargeFailure {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<SurchargeError> for SurchargeFailure {
    fn from(error: SurchargeError) -> Self {
        Self::Refused(error)
    }
}

impl IntoResponse for SurchargeFailure {
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
    path = "/api/v1/organizations/{organization_id}/surcharges",
    operation_id = "listSurcharges",
    tag = super::TAG,
    params(("organization_id" = OrganizationId, Path, description = "Organization identifier")),
    responses(
        (status = 200, description = "Every surcharge, archived ones included; the defaults are created on the first read", body = inline(DataEnvelope<Vec<SurchargeResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_surcharges(
    path: SurchargesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<SurchargeResponse>>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let surcharges = state
        .usecase
        .acting_as(user_id)
        .list_surcharges(ListSurchargesCommand {
            actor,
            organization_id: path.organization_id,
        })
        .await?;

    Ok(Response::OK(
        surcharges.into_iter().map(Into::into).collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/surcharges",
    operation_id = "createSurcharge",
    tag = super::TAG,
    params(("organization_id" = OrganizationId, Path, description = "Organization identifier")),
    request_body = SurchargeRequest,
    responses(
        (status = 201, description = "Surcharge created", body = inline(DataEnvelope<SurchargeResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 422, description = "The surcharge is refused, the field is named in details"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_surcharge(
    path: SurchargesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<SurchargeRequest>,
) -> Result<Response<SurchargeResponse>, SurchargeFailure> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let created = state
        .usecase
        .acting_as(user_id)
        .create_surcharge(CreateSurchargeCommand {
            actor,
            organization_id: path.organization_id,
            draft: payload.into(),
        })
        .await
        .map_err(ApiError::from)??;

    Ok(Response::Created(created.into()))
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/surcharges/{surcharge_id}",
    operation_id = "updateSurcharge",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("surcharge_id" = SurchargeId, Path, description = "Surcharge identifier"),
    ),
    request_body = SurchargeRequest,
    responses(
        (status = 200, description = "Surcharge replaced; periods already declared keep their rate", body = inline(DataEnvelope<SurchargeResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Surcharge not found in the organization"),
        (status = 422, description = "The surcharge is refused, the field is named in details"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_surcharge(
    path: SurchargePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<SurchargeRequest>,
) -> Result<Response<SurchargeResponse>, SurchargeFailure> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let updated = state
        .usecase
        .acting_as(user_id)
        .update_surcharge(UpdateSurchargeCommand {
            actor,
            organization_id: path.organization_id,
            id: path.surcharge_id,
            draft: payload.into(),
        })
        .await
        .map_err(ApiError::from)??;

    Ok(Response::OK(updated.into()))
}

async fn set_archived(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
    id: SurchargeId,
    archived: bool,
) -> Result<Response<SurchargeResponse>, ApiError> {
    let (user_id, actor) = resolve_actor(state, identity).await?;
    let surcharge = state
        .usecase
        .acting_as(user_id)
        .archive_surcharge(ArchiveSurchargeCommand {
            actor,
            organization_id,
            id,
            archived,
        })
        .await?;

    Ok(Response::OK(surcharge.into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/surcharges/{surcharge_id}/archive",
    operation_id = "archiveSurcharge",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("surcharge_id" = SurchargeId, Path, description = "Surcharge identifier"),
    ),
    responses(
        (status = 200, description = "Surcharge archived, past periods keep it", body = inline(DataEnvelope<SurchargeResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Surcharge not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn archive_surcharge(
    path: ArchiveSurchargePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<SurchargeResponse>, ApiError> {
    set_archived(
        &state,
        &identity,
        path.organization_id,
        path.surcharge_id,
        true,
    )
    .await
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/surcharges/{surcharge_id}/restore",
    operation_id = "restoreSurcharge",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("surcharge_id" = SurchargeId, Path, description = "Surcharge identifier"),
    ),
    responses(
        (status = 200, description = "Surcharge restored", body = inline(DataEnvelope<SurchargeResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Surcharge not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn restore_surcharge(
    path: RestoreSurchargePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<SurchargeResponse>, ApiError> {
    set_archived(
        &state,
        &identity,
        path.organization_id,
        path.surcharge_id,
        false,
    )
    .await
}

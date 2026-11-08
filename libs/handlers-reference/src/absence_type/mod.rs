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
    AbsenceKind, AbsenceType, AbsenceTypeDraft, AbsenceTypeError, AbsenceTypeId,
    ArchiveAbsenceTypeCommand, CountMode, CreateAbsenceTypeCommand, ListAbsenceTypesCommand,
    OrganizationId, UpdateAbsenceTypeCommand,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::require_org_membership;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list_types)
        .typed_post(create_type)
        .typed_put(update_type)
        .typed_post(archive_type)
        .typed_post(restore_type)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/absence-types")]
pub struct AbsenceTypesPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/absence-types/{absence_type_id}")]
pub struct AbsenceTypePath {
    pub organization_id: OrganizationId,
    pub absence_type_id: AbsenceTypeId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/absence-types/{absence_type_id}/archive")]
pub struct ArchiveAbsenceTypePath {
    pub organization_id: OrganizationId,
    pub absence_type_id: AbsenceTypeId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/absence-types/{absence_type_id}/restore")]
pub struct RestoreAbsenceTypePath {
    pub organization_id: OrganizationId,
    pub absence_type_id: AbsenceTypeId,
}

/// `account_id` is stored as given until the leave accounts exist.
#[derive(Debug, Deserialize, ToSchema)]
pub struct AbsenceTypeRequest {
    pub name: String,
    pub hue: u16,
    pub count_mode: CountMode,
    #[serde(default)]
    pub account_id: Option<Uuid>,
    pub paid: bool,
    pub needs_approval: bool,
    pub needs_proof: bool,
    pub kind: AbsenceKind,
}

impl From<AbsenceTypeRequest> for AbsenceTypeDraft {
    fn from(value: AbsenceTypeRequest) -> Self {
        Self {
            name: value.name,
            hue: value.hue,
            count_mode: value.count_mode,
            account_id: value.account_id,
            paid: value.paid,
            needs_approval: value.needs_approval,
            needs_proof: value.needs_proof,
            kind: value.kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct AbsenceTypeResponse {
    pub id: AbsenceTypeId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub hue: u16,
    pub count_mode: CountMode,
    pub account_id: Option<Uuid>,
    pub paid: bool,
    pub needs_approval: bool,
    pub needs_proof: bool,
    pub kind: AbsenceKind,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<AbsenceType> for AbsenceTypeResponse {
    fn from(value: AbsenceType) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            name: value.name.as_str().to_owned(),
            hue: value.hue.get(),
            count_mode: value.count_mode,
            account_id: value.account_id,
            paid: value.paid,
            needs_approval: value.needs_approval,
            needs_proof: value.needs_proof,
            kind: value.kind,
            archived_at: value.archived_at,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug)]
pub enum AbsenceTypeFailure {
    Api(ApiError),
    Refused(AbsenceTypeError),
}

impl From<ApiError> for AbsenceTypeFailure {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<AbsenceTypeError> for AbsenceTypeFailure {
    fn from(error: AbsenceTypeError) -> Self {
        Self::Refused(error)
    }
}

impl IntoResponse for AbsenceTypeFailure {
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
    path = "/api/v1/organizations/{organization_id}/absence-types",
    operation_id = "listAbsenceTypes",
    tag = super::TAG,
    params(("organization_id" = OrganizationId, Path, description = "Organization identifier")),
    responses(
        (status = 200, description = "Every absence type, archived ones included; the defaults are created on the first read", body = inline(DataEnvelope<Vec<AbsenceTypeResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_types(
    path: AbsenceTypesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<AbsenceTypeResponse>>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let types = state
        .usecase
        .acting_as(user_id)
        .list_absence_types(ListAbsenceTypesCommand {
            actor,
            organization_id: path.organization_id,
        })
        .await?;

    Ok(Response::OK(types.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/absence-types",
    operation_id = "createAbsenceType",
    tag = super::TAG,
    params(("organization_id" = OrganizationId, Path, description = "Organization identifier")),
    request_body = AbsenceTypeRequest,
    responses(
        (status = 201, description = "Absence type created", body = inline(DataEnvelope<AbsenceTypeResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 422, description = "The type is refused, the field is named in details"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_type(
    path: AbsenceTypesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<AbsenceTypeRequest>,
) -> Result<Response<AbsenceTypeResponse>, AbsenceTypeFailure> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let created = state
        .usecase
        .acting_as(user_id)
        .create_absence_type(CreateAbsenceTypeCommand {
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
    path = "/api/v1/organizations/{organization_id}/absence-types/{absence_type_id}",
    operation_id = "updateAbsenceType",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("absence_type_id" = AbsenceTypeId, Path, description = "Absence type identifier"),
    ),
    request_body = AbsenceTypeRequest,
    responses(
        (status = 200, description = "Absence type replaced", body = inline(DataEnvelope<AbsenceTypeResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Absence type not found in the organization"),
        (status = 422, description = "The type is refused, the field is named in details"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_type(
    path: AbsenceTypePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<AbsenceTypeRequest>,
) -> Result<Response<AbsenceTypeResponse>, AbsenceTypeFailure> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let updated = state
        .usecase
        .acting_as(user_id)
        .update_absence_type(UpdateAbsenceTypeCommand {
            actor,
            organization_id: path.organization_id,
            id: path.absence_type_id,
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
    id: AbsenceTypeId,
    archived: bool,
) -> Result<Response<AbsenceTypeResponse>, ApiError> {
    let (user_id, actor) = resolve_actor(state, identity).await?;
    let absence_type = state
        .usecase
        .acting_as(user_id)
        .archive_absence_type(ArchiveAbsenceTypeCommand {
            actor,
            organization_id,
            id,
            archived,
        })
        .await?;

    Ok(Response::OK(absence_type.into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/absence-types/{absence_type_id}/archive",
    operation_id = "archiveAbsenceType",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("absence_type_id" = AbsenceTypeId, Path, description = "Absence type identifier"),
    ),
    responses(
        (status = 200, description = "Absence type archived, past absences keep it", body = inline(DataEnvelope<AbsenceTypeResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Absence type not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn archive_type(
    path: ArchiveAbsenceTypePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<AbsenceTypeResponse>, ApiError> {
    set_archived(
        &state,
        &identity,
        path.organization_id,
        path.absence_type_id,
        true,
    )
    .await
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/absence-types/{absence_type_id}/restore",
    operation_id = "restoreAbsenceType",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("absence_type_id" = AbsenceTypeId, Path, description = "Absence type identifier"),
    ),
    responses(
        (status = 200, description = "Absence type restored", body = inline(DataEnvelope<AbsenceTypeResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Absence type not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn restore_type(
    path: RestoreAbsenceTypePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<AbsenceTypeResponse>, ApiError> {
    set_archived(
        &state,
        &identity,
        path.organization_id,
        path.absence_type_id,
        false,
    )
    .await
}

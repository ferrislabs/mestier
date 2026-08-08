use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::{AbsenceKind, CreateAbsenceCommand, EmployeeId};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    absence::{AbsenceResponse, AbsencesPath, require_employee_target},
    require_org_membership,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAbsenceRequest {
    pub employee_id: EmployeeId,
    pub kind: AbsenceKind,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    #[serde(default)]
    pub all_day: bool,
    #[serde(default)]
    pub note: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/absences",
    operation_id = "createAbsence",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateAbsenceRequest,
    responses(
        (status = 201, description = "Absence created", body = inline(DataEnvelope<AbsenceResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Absence conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: AbsencesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateAbsenceRequest>,
) -> Result<Response<AbsenceResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    require_employee_target(&state, path.organization_id, payload.employee_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;

    let absence = state
        .usecase
        .acting_as(actor)
        .create_absence(CreateAbsenceCommand {
            organization_id: path.organization_id,
            employee_id: payload.employee_id,
            kind: payload.kind,
            starts_at: payload.starts_at,
            ends_at: payload.ends_at,
            all_day: payload.all_day,
            note: payload.note,
        })
        .await?;

    Ok(Response::Created(absence.into()))
}

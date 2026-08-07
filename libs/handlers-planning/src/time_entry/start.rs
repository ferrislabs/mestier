use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{EmployeeId, StartTimeEntryCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::time_entry::{
    StartTimeEntryPath, TimeEntryResponse, require_employee_access, resolve_caller_user,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct StartTimeEntryRequest {
    /// Target employee. When omitted, resolved from the caller's linked
    /// employee record in this organization.
    #[serde(default)]
    pub employee_id: Option<EmployeeId>,
    /// Opaque keys previously returned by `POST /api/v1/files`.
    #[serde(default)]
    pub photos_before: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/work-orders/{work_order_id}/time-entries/start",
    operation_id = "startTimeEntry",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("work_order_id" = mestier_core::WorkOrderId, Path, description = "Work order identifier"),
    ),
    request_body = StartTimeEntryRequest,
    responses(
        (status = 201, description = "Time entry started", body = inline(DataEnvelope<TimeEntryResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 409, description = "Open time entry already exists or employee not assigned"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: StartTimeEntryPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<StartTimeEntryRequest>,
) -> Result<Response<TimeEntryResponse>, ApiError> {
    let (_user_id, caller_employee_id) =
        resolve_caller_user(&state, &identity, path.organization_id).await?;

    let employee_id = match payload.employee_id {
        Some(employee_id) => employee_id,
        None => caller_employee_id.ok_or_else(|| {
            ApiError::Validation(
                "employee_id is required when the caller has no linked employee".to_owned(),
            )
        })?,
    };

    require_employee_access(&state, &identity, path.organization_id, employee_id).await?;

    let time_entry = state
        .usecase
        .start_time_entry(StartTimeEntryCommand {
            organization_id: path.organization_id,
            work_order_id: path.work_order_id,
            employee_id,
            photos_before: payload.photos_before,
        })
        .await?;

    Ok(Response::Created(time_entry.into()))
}

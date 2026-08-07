use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::EmployeeId;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::time_entry::{ActiveTimeEntryPath, TimeEntryResponse, require_employee_access, resolve_caller_user};

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct ActiveTimeEntryQuery {
    /// Target employee. When omitted, resolved from the caller's linked
    /// employee record in this organization.
    pub employee_id: Option<EmployeeId>,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/time-entries/active",
    operation_id = "getActiveTimeEntry",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ActiveTimeEntryQuery,
    ),
    responses(
        (status = 200, description = "Active time entry, if any", body = inline(DataEnvelope<Option<TimeEntryResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ActiveTimeEntryPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    axum::extract::Query(query): axum::extract::Query<ActiveTimeEntryQuery>,
) -> Result<Response<Option<TimeEntryResponse>>, ApiError> {
    let (_user_id, caller_employee_id) =
        resolve_caller_user(&state, &identity, path.organization_id).await?;

    let employee_id = match query.employee_id {
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
        .get_active_time_entry(path.organization_id, employee_id)
        .await?;

    Ok(Response::OK(time_entry.map(Into::into)))
}

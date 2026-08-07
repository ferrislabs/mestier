use auth::Identity;
use axum::{
    Extension, Json,
    extract::{Query, State},
};
use chrono::NaiveDate;
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{ReplaceWorkSlotsCommand, WorkSlotInput};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    require_org_membership,
    work_time::{WorkSlotResponse, WorkSlotsPath, WorkTimeWindowQuery},
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct WorkSlotRequest {
    pub work_date: NaiveDate,
    pub starts_minute: i16,
    pub ends_minute: i16,
}

/// The complete replacement set for the `[from, to]` window — never a
/// delta, mirroring the `PATCH work-orders` `assignees` contract.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PutWorkSlotsRequest {
    pub slots: Vec<WorkSlotRequest>,
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/employees/{employee_id}/work-slots",
    operation_id = "putWorkSlots",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("employee_id" = mestier_core::EmployeeId, Path, description = "Employee identifier"),
        ("from" = chrono::NaiveDate, Query, description = "Window start (inclusive)"),
        ("to" = chrono::NaiveDate, Query, description = "Window end (inclusive)"),
    ),
    request_body = PutWorkSlotsRequest,
    responses(
        (status = 200, description = "Work slots window replaced", body = inline(DataEnvelope<Vec<WorkSlotResponse>>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Work slots conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    WorkSlotsPath {
        organization_id,
        employee_id,
    }: WorkSlotsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(window): Query<WorkTimeWindowQuery>,
    Json(payload): Json<PutWorkSlotsRequest>,
) -> Result<Response<Vec<WorkSlotResponse>>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    crate::absence::require_employee_target(&state, organization_id, employee_id).await?;

    let work_slots = state
        .usecase
        .replace_work_slots(ReplaceWorkSlotsCommand {
            organization_id,
            employee_id,
            from: window.from,
            to: window.to,
            slots: payload
                .slots
                .into_iter()
                .map(|slot| WorkSlotInput {
                    work_date: slot.work_date,
                    starts_minute: slot.starts_minute,
                    ends_minute: slot.ends_minute,
                })
                .collect(),
        })
        .await?;

    Ok(Response::OK(
        work_slots.into_iter().map(WorkSlotResponse::from).collect(),
    ))
}

use auth::Identity;
use axum::{
    Extension, Json,
    extract::{Query, State},
};
use chrono::NaiveDate;
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::{ReplaceWorkSlotsCommand, WorkSlotInput};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::work_time::{WorkSlotResponse, WorkSlotsPath, WorkTimeWindowQuery};

#[derive(Debug, Deserialize, ToSchema)]
pub struct WorkSlotRequest {
    pub work_date: NaiveDate,
    pub starts_minute: i16,
    pub ends_minute: i16,
}

/// The complete replacement set for the `[from, to]` window — never a
/// delta, mirroring the `PATCH tasks` `assignees` contract.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PutWorkSlotsRequest {
    pub slots: Vec<WorkSlotRequest>,
}

#[utoipa::path(
    put,
    path = "/api/v1/members/{member_id}/work-slots",
    operation_id = "putWorkSlots",
    tag = super::super::TAG,
    params(
        ("member_id" = mestier_core::MemberId, Path, description = "Member identifier"),
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
    WorkSlotsPath { member_id }: WorkSlotsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(window): Query<WorkTimeWindowQuery>,
    Json(payload): Json<PutWorkSlotsRequest>,
) -> Result<Response<Vec<WorkSlotResponse>>, ApiError> {
    let actor = resolve_user_id(&state, &identity).await?;
    let organization_id = crate::require_member_target(&state, &identity, member_id).await?;

    let work_slots = state
        .usecase
        .acting_as(actor)
        .replace_work_slots(ReplaceWorkSlotsCommand {
            organization_id,
            member_id,
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

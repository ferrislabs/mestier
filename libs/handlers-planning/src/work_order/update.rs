use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{AssigneeRef, EmployeeId, PatchWorkOrderCommand, UserId, WorkOrderStatus};
use serde::{Deserialize, Deserializer};
use utoipa::ToSchema;

use crate::{
    response::PatchWorkOrderResponse,
    work_order::{WorkOrderPath, require_work_order},
};

/// Distinguishes "the key is absent" (leave the field unchanged) from "the
/// key is present" (apply it, `null` included) — plain `Option<Option<T>>`
/// cannot make that distinction on its own because serde treats a missing
/// key and an explicit `null` the same way by default.
fn deserialize_present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AssigneeRefRequest {
    Employee { employee_id: EmployeeId },
    Member { user_id: UserId },
}

impl From<AssigneeRefRequest> for AssigneeRef {
    fn from(value: AssigneeRefRequest) -> Self {
        match value {
            AssigneeRefRequest::Employee { employee_id } => AssigneeRef::Employee(employee_id),
            AssigneeRefRequest::Member { user_id } => AssigneeRef::Member(user_id),
        }
    }
}

/// Every field is optional and, when present, replaces the current value —
/// `title`/`note` additionally distinguish "absent" from "present but
/// `null`" (see [`deserialize_present`]), and `assignees` is the complete
/// replacement list, never a delta.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateWorkOrderRequest {
    #[serde(default)]
    pub starts_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub ends_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub all_day: Option<bool>,
    #[serde(default)]
    pub status: Option<WorkOrderStatus>,
    #[serde(default, deserialize_with = "deserialize_present")]
    #[schema(value_type = Option<String>, nullable)]
    pub title: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    #[schema(value_type = Option<String>, nullable)]
    pub note: Option<Option<String>>,
    #[serde(default)]
    pub assignees: Option<Vec<AssigneeRefRequest>>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/work-orders/{work_order_id}",
    operation_id = "patchWorkOrder",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("work_order_id" = mestier_core::WorkOrderId, Path, description = "Work order identifier"),
    ),
    request_body = UpdateWorkOrderRequest,
    responses(
        (status = 200, description = "Work order rescheduled and/or reassigned", body = inline(DataEnvelope<PatchWorkOrderResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Work order, employee or member not found"),
        (status = 409, description = "Work order conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    WorkOrderPath {
        organization_id,
        work_order_id,
    }: WorkOrderPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateWorkOrderRequest>,
) -> Result<Response<PatchWorkOrderResponse>, ApiError> {
    require_work_order(&state, &identity, organization_id, work_order_id).await?;

    let mut command = PatchWorkOrderCommand::new(work_order_id);
    command.starts_at = payload.starts_at;
    command.ends_at = payload.ends_at;
    command.all_day = payload.all_day;
    command.status = payload.status;
    command.title = payload.title;
    command.note = payload.note;
    command.assignees = payload
        .assignees
        .map(|assignees| assignees.into_iter().map(Into::into).collect());

    let (work_order, created_employees) = state.usecase.patch_work_order(command).await?;

    Ok(Response::OK(PatchWorkOrderResponse {
        work_order: work_order.into(),
        created_employees: created_employees.into_iter().map(Into::into).collect(),
    }))
}

// No `#[cfg(test)]` module here: exercising `UpdateWorkOrderRequest`'s JSON
// deserialization (the double-option shim, the tagged `assignees` union)
// would need `serde_json` as a dev-dependency, which is not among
// `handlers-planning`'s current `Cargo.toml` dependencies (a file this
// workstream does not own — see the report's convergence notes). No other
// `handlers-*` crate in the repo unit-tests request deserialization either;
// the semantics `deserialize_present` protects — "absent leaves the field
// unchanged" vs. "present-but-null clears it" — are covered at the domain
// layer by `WorkOrderService`'s `patch_work_order_*` tests
// (`libs/core/src/domain/work_order/service.rs`), which exercise
// `PatchWorkOrderCommand`'s `Option<Option<String>>` fields directly.

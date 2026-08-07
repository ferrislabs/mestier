use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{
    AssigneeRef, EmployeeId, EquipmentId, PatchWorkOrderCommand, UserId, WorkOrderStatus,
};
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

#[derive(Debug, PartialEq, Deserialize, ToSchema)]
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
/// `null`" (see [`deserialize_present`]), and `assignees` / `equipment` are
/// the complete replacement lists, never a delta.
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
    #[serde(default)]
    pub equipment: Option<Vec<EquipmentId>>,
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
    command.equipment = payload.equipment;

    let (work_order, created_employees) = state.usecase.patch_work_order(command).await?;

    Ok(Response::OK(PatchWorkOrderResponse {
        work_order: work_order.into(),
        created_employees: created_employees.into_iter().map(Into::into).collect(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse(value: serde_json::Value) -> UpdateWorkOrderRequest {
        serde_json::from_value(value).expect("payload must deserialize")
    }

    // ── absent vs. `null` on `title`/`note` ─────────────────────────────

    #[test]
    fn absent_title_and_note_leave_both_fields_unset() {
        let request = parse(json!({}));

        assert_eq!(
            request.title, None,
            "an absent `title` key must not be mistaken for a clearing `null`"
        );
        assert_eq!(
            request.note, None,
            "an absent `note` key must not be mistaken for a clearing `null`"
        );
    }

    #[test]
    fn null_title_clears_it() {
        let request = parse(json!({ "title": null }));

        assert_eq!(
            request.title,
            Some(None),
            "an explicit `null` must be distinguishable from an absent key: it clears the title"
        );
    }

    #[test]
    fn null_note_clears_it() {
        let request = parse(json!({ "note": null }));

        assert_eq!(
            request.note,
            Some(None),
            "the note field must clear independently of the title field"
        );
    }

    #[test]
    fn present_title_sets_the_new_value() {
        let request = parse(json!({ "title": "Nouveau titre" }));

        assert_eq!(request.title, Some(Some("Nouveau titre".to_owned())));
    }

    // ── the tagged `assignees` union ────────────────────────────────────

    #[test]
    fn employee_kind_deserializes_to_the_employee_variant() {
        let employee_id: EmployeeId = "11111111-1111-1111-1111-111111111111".parse().unwrap();
        let parsed: AssigneeRefRequest = serde_json::from_value(json!({
            "kind": "employee",
            "employee_id": employee_id.0.to_string(),
        }))
        .expect("an employee assignee must deserialize");

        match AssigneeRef::from(parsed) {
            AssigneeRef::Employee(id) => assert_eq!(id, employee_id),
            other => panic!("expected AssigneeRef::Employee, got {other:?}"),
        }
    }

    #[test]
    fn member_kind_deserializes_to_the_member_variant() {
        let user_id: UserId = "22222222-2222-2222-2222-222222222222".parse().unwrap();
        let parsed: AssigneeRefRequest = serde_json::from_value(json!({
            "kind": "member",
            "user_id": user_id.0.to_string(),
        }))
        .expect("a member assignee must deserialize");

        match AssigneeRef::from(parsed) {
            AssigneeRef::Member(id) => assert_eq!(id, user_id),
            other => panic!("expected AssigneeRef::Member, got {other:?}"),
        }
    }

    #[test]
    fn unknown_assignee_kind_is_rejected_not_ignored() {
        let result: Result<AssigneeRefRequest, _> = serde_json::from_value(json!({
            "kind": "equipment",
            "equipment_id": "33333333-3333-3333-3333-333333333333",
        }));

        assert!(
            result.is_err(),
            "an unrecognized `kind` must fail to deserialize rather than being silently dropped"
        );
    }

    // ── empty vs. absent `assignees` ────────────────────────────────────

    #[test]
    fn empty_assignees_array_means_clear_every_assignment() {
        let request = parse(json!({ "assignees": [] }));

        assert_eq!(
            request.assignees,
            Some(Vec::new()),
            "`assignees: []` is a present, empty replacement list — it must drop every \
             assignment, not be mistaken for \"don't touch assignments\""
        );
    }

    #[test]
    fn absent_assignees_means_leave_assignments_untouched() {
        let request = parse(json!({}));

        assert_eq!(
            request.assignees, None,
            "an absent `assignees` key must leave current assignments untouched, which is a \
             different outcome than `assignees: []`"
        );
        assert_eq!(
            request.equipment, None,
            "an absent `equipment` key must leave current equipment untouched"
        );
    }

    #[test]
    fn empty_equipment_array_means_clear_every_link() {
        let request = parse(json!({ "equipment": [] }));

        assert_eq!(
            request.equipment,
            Some(Vec::new()),
            "`equipment: []` is a present, empty replacement list — it must drop every \
             equipment link"
        );
    }

    #[test]
    fn assignees_carries_both_kinds_together() {
        let employee_id: EmployeeId = "11111111-1111-1111-1111-111111111111".parse().unwrap();
        let user_id: UserId = "22222222-2222-2222-2222-222222222222".parse().unwrap();
        let request = parse(json!({
            "assignees": [
                { "kind": "employee", "employee_id": employee_id.0.to_string() },
                { "kind": "member", "user_id": user_id.0.to_string() },
            ]
        }));

        let assignees = request.assignees.expect("assignees must be present");
        assert_eq!(assignees.len(), 2);

        let resolved: Vec<AssigneeRef> = assignees.into_iter().map(Into::into).collect();
        assert!(matches!(resolved[0], AssigneeRef::Employee(id) if id == employee_id));
        assert!(matches!(resolved[1], AssigneeRef::Member(id) if id == user_id));
    }
}

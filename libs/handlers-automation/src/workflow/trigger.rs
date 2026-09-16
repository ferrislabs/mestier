//! `GET`/`PUT` a workflow's trigger (#225): which event(s) start a run for
//! it. The Start node of the workflow editor (#204) calls this once an
//! event is picked from the catalogue (`GET .../automation/events`).

use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{SetWorkflowTriggerCommand, WorkflowTrigger};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::WorkflowTriggerPath,
    require_manage_automation, require_view_automation,
    response::{WorkflowTriggerModeDto, WorkflowTriggerResponse},
    workflow::find_workflow_in_org,
};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}/trigger",
    operation_id = "getWorkflowTrigger",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("workflow_id" = uuid::Uuid, Path, description = "Workflow identifier"),
    ),
    responses(
        (status = 200, description = "The event(s) this workflow currently triggers from, empty when it has no subscription", body = inline(DataEnvelope<WorkflowTriggerResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Workflow not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_trigger(
    WorkflowTriggerPath {
        organization_id,
        workflow_id,
    }: WorkflowTriggerPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<WorkflowTriggerResponse>, ApiError> {
    require_view_automation(&state, &identity, organization_id).await?;
    find_workflow_in_org(&state, organization_id, workflow_id).await?;

    let trigger = state
        .usecase
        .workflow_trigger(organization_id, workflow_id)
        .await?;

    Ok(Response::OK(WorkflowTriggerResponse::from(trigger)))
}

/// Always the full desired selection — replaces whatever was there, never an
/// addition to it. An empty list clears the trigger.
#[derive(Debug, Deserialize, ToSchema)]
pub struct SetWorkflowTriggerRequest {
    pub mode: WorkflowTriggerModeDto,
    pub event_names: Vec<String>,
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}/trigger",
    operation_id = "setWorkflowTrigger",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("workflow_id" = uuid::Uuid, Path, description = "Workflow identifier"),
    ),
    request_body = SetWorkflowTriggerRequest,
    responses(
        (status = 200, description = "Trigger replaced (or cleared, for an empty selection)", body = inline(DataEnvelope<WorkflowTriggerResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Workflow not found"),
        (status = 409, description = "An event name outside the catalogue"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn set_trigger(
    WorkflowTriggerPath {
        organization_id,
        workflow_id,
    }: WorkflowTriggerPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<SetWorkflowTriggerRequest>,
) -> Result<Response<WorkflowTriggerResponse>, ApiError> {
    require_manage_automation(&state, &identity, organization_id).await?;
    find_workflow_in_org(&state, organization_id, workflow_id).await?;

    let trigger = match payload.mode {
        WorkflowTriggerModeDto::Events => WorkflowTrigger::Events(payload.event_names),
        WorkflowTriggerModeDto::Manual => WorkflowTrigger::Manual,
    };
    let trigger = state
        .usecase
        .set_workflow_trigger(SetWorkflowTriggerCommand {
            org_id: organization_id,
            workflow_id,
            trigger,
        })
        .await?;

    Ok(Response::OK(WorkflowTriggerResponse::from(trigger)))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn a_list_of_event_names_parses() {
        let request: SetWorkflowTriggerRequest =
            serde_json::from_value(json!({ "mode": "events", "event_names": ["quote.accepted"] }))
                .expect("payload must deserialize");

        assert_eq!(request.mode, WorkflowTriggerModeDto::Events);
        assert_eq!(request.event_names, vec!["quote.accepted".to_string()]);
    }

    #[test]
    fn an_empty_list_parses_and_clears_the_trigger() {
        let request: SetWorkflowTriggerRequest =
            serde_json::from_value(json!({ "mode": "events", "event_names": [] }))
                .expect("payload must deserialize");

        assert_eq!(request.mode, WorkflowTriggerModeDto::Events);
        assert!(request.event_names.is_empty());
    }

    #[test]
    fn a_manual_mode_parses_regardless_of_event_names() {
        let request: SetWorkflowTriggerRequest =
            serde_json::from_value(json!({ "mode": "manual", "event_names": ["quote.accepted"] }))
                .expect("payload must deserialize");

        assert_eq!(request.mode, WorkflowTriggerModeDto::Manual);
    }
}

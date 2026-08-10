use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::UpdateWorkflowCommand;
use serde::{Deserialize, Deserializer};
use utoipa::ToSchema;

use crate::{paths::WorkflowPath, response::WorkflowResponse, workflow::require_workflow};

/// Distinguishes "the key is absent" (leave the field unchanged) from "the
/// key is present" (apply it, `null` included) — same convention as
/// `handlers-planning::task::update::deserialize_present`.
fn deserialize_present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}

/// Every field optional. `enabled` is #203's "enable/disable"; `name` and
/// `description` come along for free since `UpdateWorkflowCommand` already
/// carries them.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateWorkflowRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_present")]
    #[schema(value_type = Option<String>, nullable)]
    pub description: Option<Option<String>>,
    #[serde(default)]
    pub enabled: Option<bool>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}",
    operation_id = "updateWorkflow",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("workflow_id" = uuid::Uuid, Path, description = "Workflow identifier"),
    ),
    request_body = UpdateWorkflowRequest,
    responses(
        (status = 200, description = "Workflow updated", body = inline(DataEnvelope<WorkflowResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Workflow not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    WorkflowPath {
        organization_id,
        workflow_id,
    }: WorkflowPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateWorkflowRequest>,
) -> Result<Response<WorkflowResponse>, ApiError> {
    require_workflow(&state, &identity, organization_id, workflow_id).await?;

    let updated = state
        .usecase
        .update_workflow(UpdateWorkflowCommand {
            org_id: organization_id,
            id: workflow_id,
            name: payload.name,
            description: payload.description,
            enabled: payload.enabled,
        })
        .await?;

    Ok(Response::OK(WorkflowResponse::from(updated)))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn parse(value: serde_json::Value) -> UpdateWorkflowRequest {
        serde_json::from_value(value).expect("payload must deserialize")
    }

    #[test]
    fn absent_fields_leave_them_unset() {
        let request = parse(json!({}));

        assert_eq!(request.name, None);
        assert_eq!(request.description, None);
        assert_eq!(request.enabled, None);
    }

    #[test]
    fn null_description_clears_it() {
        let request = parse(json!({ "description": null }));

        assert_eq!(request.description, Some(None));
    }

    #[test]
    fn present_description_sets_it() {
        let request = parse(json!({ "description": "New description" }));

        assert_eq!(
            request.description,
            Some(Some("New description".to_owned()))
        );
    }

    #[test]
    fn enabled_false_disables_the_workflow() {
        let request = parse(json!({ "enabled": false }));

        assert_eq!(request.enabled, Some(false));
    }
}

use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{AppState, DataEnvelope, Response};
use mestier_core::{UpdateWorkflowCommand, WorkflowDescription, WorkflowName, WorkflowTags};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    error::AutomationError,
    paths::WorkflowPath,
    request::{deserialize_present, invalid},
    require_manage_automation,
    response::WorkflowResponse,
    workflow::find_workflow_in_org,
};

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
    #[serde(default, deserialize_with = "deserialize_present")]
    #[schema(value_type = Option<Uuid>, nullable)]
    pub folder_id: Option<Option<Uuid>>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub archived: Option<bool>,
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
        (status = 200, description = "Workflow updated. `archived: true` also disables it; `archived: false` leaves it disabled. updated_at only moves for a new name or description", body = inline(DataEnvelope<WorkflowResponse>)),
        (status = 400, description = "Validation failed: name empty or over 120 characters, description over 2000 characters, more than 10 tags or a tag outside 1 to 30 characters, or archived and enabled both true in one request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Workflow not found"),
        (status = 409, description = "E_WORKFLOW_ARCHIVED when enabling an archived workflow; otherwise the folder does not exist in this organization"),
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
) -> Result<Response<WorkflowResponse>, AutomationError> {
    require_manage_automation(&state, &identity, organization_id).await?;
    find_workflow_in_org(&state, organization_id, workflow_id).await?;

    let name = payload
        .name
        .map(WorkflowName::try_from)
        .transpose()
        .map_err(invalid)?;
    let description = payload
        .description
        .map(|text| text.map(WorkflowDescription::try_from).transpose())
        .transpose()
        .map_err(invalid)?;
    let tags = payload
        .tags
        .map(WorkflowTags::try_from)
        .transpose()
        .map_err(invalid)?;
    let updated = state
        .usecase
        .update_workflow(UpdateWorkflowCommand {
            org_id: organization_id,
            id: workflow_id,
            name,
            description,
            enabled: payload.enabled,
            folder_id: payload.folder_id,
            tags,
            archived: payload.archived,
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

    #[test]
    fn absent_classification_fields_leave_them_unset() {
        let request = parse(json!({}));

        assert_eq!(request.folder_id, None);
        assert_eq!(request.tags, None);
        assert_eq!(request.archived, None);
    }

    #[test]
    fn a_null_folder_sends_the_workflow_to_the_root() {
        let request = parse(json!({ "folder_id": null }));

        assert_eq!(request.folder_id, Some(None));
    }

    #[test]
    fn a_present_folder_is_kept() {
        let folder = Uuid::from_u128(9);

        let request = parse(json!({ "folder_id": folder }));

        assert_eq!(request.folder_id, Some(Some(folder)));
    }

    #[test]
    fn tags_and_archived_are_read_as_given() {
        let request = parse(json!({ "tags": ["crm", "relance"], "archived": true }));

        assert_eq!(
            request.tags,
            Some(vec!["crm".to_owned(), "relance".to_owned()])
        );
        assert_eq!(request.archived, Some(true));
    }
}

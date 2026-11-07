use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{FolderName, UpdateFolderCommand};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    paths::FolderPath, request::deserialize_present, require_manage_automation,
    response::FolderResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateFolderRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_present")]
    #[schema(value_type = Option<Uuid>, nullable)]
    pub parent_id: Option<Option<Uuid>>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/automation/folders/{folder_id}",
    operation_id = "updateWorkflowFolder",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("folder_id" = uuid::Uuid, Path, description = "Folder identifier"),
    ),
    request_body = UpdateFolderRequest,
    responses(
        (status = 200, description = "Folder renamed and/or moved; a null parent_id moves it to the root", body = inline(DataEnvelope<FolderResponse>)),
        (status = 400, description = "The name is empty or longer than 80 characters"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Folder not found"),
        (status = 409, description = "The new parent does not exist, would close a loop, would push the folder or its sub-folders past 8 levels, or a sibling already has this name (case-insensitive)"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    FolderPath {
        organization_id,
        folder_id,
    }: FolderPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateFolderRequest>,
) -> Result<Response<FolderResponse>, ApiError> {
    require_manage_automation(&state, &identity, organization_id).await?;

    let name = payload
        .name
        .map(FolderName::try_from)
        .transpose()
        .map_err(|error| ApiError::Validation(error.to_string()))?;
    let folder = state
        .usecase
        .update_workflow_folder(UpdateFolderCommand {
            org_id: organization_id,
            id: folder_id,
            name,
            parent_id: payload.parent_id,
        })
        .await?;

    Ok(Response::OK(FolderResponse::from(folder)))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn parse(value: serde_json::Value) -> UpdateFolderRequest {
        serde_json::from_value(value).expect("payload must deserialize")
    }

    #[test]
    fn absent_fields_leave_them_unset() {
        let request = parse(json!({}));

        assert_eq!(request.name, None);
        assert_eq!(request.parent_id, None);
    }

    #[test]
    fn a_null_parent_means_the_root() {
        assert_eq!(parse(json!({ "parent_id": null })).parent_id, Some(None));
    }

    #[test]
    fn a_present_parent_is_kept() {
        let parent = Uuid::from_u128(7);

        let request = parse(json!({ "parent_id": parent }));

        assert_eq!(request.parent_id, Some(Some(parent)));
    }
}

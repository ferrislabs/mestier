use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, require_org_membership, resolve_actor};
use mestier_core::{ProjectId, UpdateLibraryPhotoCommand};
use serde::{Deserialize, Deserializer};
use utoipa::ToSchema;

use crate::photo_library::{PhotoPath, response::LibraryPhotoResponse, unprocessable};

fn present<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateLibraryPhotoRequest {
    /// An empty string clears the caption. Absent leaves it unchanged.
    #[serde(default)]
    pub caption: Option<String>,
    /// Replaces the whole list. Trimmed, lowercased and deduplicated; at most
    /// 20 tags of 1 to 40 characters. Absent leaves the list unchanged.
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    /// `null` clears the project. Absent leaves it unchanged.
    #[serde(default, deserialize_with = "present")]
    #[schema(value_type = Option<ProjectId>, nullable)]
    pub project_id: Option<Option<ProjectId>>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/photo-library/photos/{photo_id}",
    operation_id = "updateLibraryPhoto",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("photo_id" = mestier_core::LibraryPhotoId, Path, description = "Photo identifier"),
    ),
    request_body = UpdateLibraryPhotoRequest,
    responses(
        (status = 200, description = "The photo, updated", body = inline(DataEnvelope<LibraryPhotoResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Photo or project not found, or held by another organization"),
        (status = 422, description = "An empty or too long tag, or more than 20 tags"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    PhotoPath {
        organization_id,
        photo_id,
    }: PhotoPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateLibraryPhotoRequest>,
) -> Result<Response<LibraryPhotoResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let photo = state
        .usecase
        .acting_as(user_id)
        .update_library_photo(UpdateLibraryPhotoCommand {
            actor,
            organization_id,
            id: photo_id,
            caption: payload.caption,
            tags: payload.tags,
            project_id: payload.project_id,
        })
        .await?
        .map_err(unprocessable)?;

    Ok(Response::OK(photo.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> UpdateLibraryPhotoRequest {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn absent_fields_stay_absent() {
        let request = parse("{}");

        assert_eq!(request.caption, None);
        assert_eq!(request.tags, None);
        assert_eq!(request.project_id, None);
    }

    #[test]
    fn a_null_project_is_told_apart_from_an_absent_one() {
        assert_eq!(parse(r#"{"project_id": null}"#).project_id, Some(None));
        assert!(matches!(
            parse(r#"{"project_id": "0192f0a0-0000-7000-8000-000000000001"}"#).project_id,
            Some(Some(_))
        ));
    }

    #[test]
    fn an_empty_caption_is_kept_so_it_can_clear() {
        assert_eq!(parse(r#"{"caption": ""}"#).caption.as_deref(), Some(""));
    }
}

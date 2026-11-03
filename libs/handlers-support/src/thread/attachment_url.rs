use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::AttachmentUrlPath, require_view_support, response::AttachmentUrlResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/support/attachments/{attachment_id}/url",
    operation_id = "getSupportAttachmentUrl",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("attachment_id" = mestier_core::AttachmentId, Path, description = "Attachment identifier"),
    ),
    responses(
        (status = 200, description = "A short-lived presigned read url", body = inline(DataEnvelope<AttachmentUrlResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Attachment not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    AttachmentUrlPath {
        organization_id,
        attachment_id,
    }: AttachmentUrlPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<AttachmentUrlResponse>, ApiError> {
    require_view_support(&state, &identity, organization_id).await?;

    let located = state
        .usecase
        .get_support_attachment(organization_id, attachment_id)
        .await?;
    let presigned = state
        .file_storage
        .presigned_get_url(&located.attachment.file_key)
        .await?;

    Ok(Response::OK(presigned.into()))
}

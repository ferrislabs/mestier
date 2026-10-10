use auth::Identity;
use axum::{
    Extension, Json,
    body::Body,
    extract::State,
    response::{IntoResponse, Response},
};
use handlers::{ApiError, AppState, resolve_actor};
use http::{StatusCode, header::CONTENT_TYPE};
use mestier_core::{DoeStyle, PreviewDoeStyleCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    doe::pdf::{load_files, render_off_the_executor},
    doe_model::DoeStylePreviewPath,
    require_org_membership,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct PreviewDoeStyleRequest {
    pub style: DoeStyle,
}

/// Lays a made-up DOE out with a style that is not saved, so the style can
/// be judged before it is. Shares the export's render limits.
#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/doe-style/preview",
    operation_id = "previewDoeStyle",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = PreviewDoeStyleRequest,
    responses(
        (status = 200, description = "A sample DOE in this style, as a PDF; nothing is saved", content_type = "application/pdf"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "The logo is no longer in storage"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: DoeStylePreviewPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<PreviewDoeStyleRequest>,
) -> Result<Response, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let document = state
        .usecase
        .acting_as(user_id)
        .preview_doe_style(PreviewDoeStyleCommand {
            actor,
            organization_id: path.organization_id,
            style: payload.style,
        })
        .await?;
    let files = load_files(&state, &document).await?;
    let bytes = render_off_the_executor(document, files).await?;

    Ok((
        StatusCode::OK,
        [(CONTENT_TYPE, "application/pdf")],
        Body::from(bytes),
    )
        .into_response())
}

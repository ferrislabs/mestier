use axum::extract::{Query, State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{paths::FileUrlPath, response::FileUrlResponse};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct FileUrlQuery {
    /// The storage key returned by the upload endpoint.
    pub key: String,
}

/// Hands back a short-lived link the browser loads the object from directly.
///
/// Authorization is the caller holding the key. Keys are unguessable v7 uuids
/// and are only ever disclosed inside an organization-scoped payload, such as
/// a quote line's `photo_keys`, so learning one already means being allowed to
/// see the record that carries it. That is a capability model, not ownership:
/// nothing here would stop a caller who obtained a key by other means. Closing
/// that needs a files table recording who owns each object, which is a wider
/// change than this endpoint.
#[utoipa::path(
    get,
    path = "/api/v1/files/url",
    operation_id = "getFileUrl",
    tag = crate::TAG,
    params(FileUrlQuery),
    responses(
        (status = 200, description = "Presigned read url", body = inline(DataEnvelope<FileUrlResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 409, description = "Invalid file key"),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, err)]
pub async fn handler(
    _: FileUrlPath,
    State(state): State<AppState>,
    Query(query): Query<FileUrlQuery>,
) -> Result<Response<FileUrlResponse>, ApiError> {
    let presigned = state.file_storage.presigned_get_url(&query.key).await?;

    Ok(Response::OK(presigned.into()))
}

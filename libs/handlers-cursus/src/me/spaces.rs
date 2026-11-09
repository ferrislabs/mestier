use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};

use crate::{paths::MySpacesPath, response::CursusSpaceResponse};

#[utoipa::path(
    get,
    path = "/api/v1/cursus/me/spaces",
    operation_id = "listMyCursusSpaces",
    tag = crate::TAG,
    responses(
        (status = 200, description = "Organizations where the caller is an active participant", body = inline(DataEnvelope<Vec<CursusSpaceResponse>>)),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    _: MySpacesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<CursusSpaceResponse>>, ApiError> {
    let user_id = resolve_user_id(&state, &identity).await?;

    let spaces = state.usecase.list_my_cursus_spaces(user_id).await?;

    Ok(Response::OK(
        spaces.into_iter().map(CursusSpaceResponse::from).collect(),
    ))
}

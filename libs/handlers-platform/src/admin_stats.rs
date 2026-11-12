use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};

use crate::{AdminStatsPath, platform_admin, response::PlatformStatsResponse};

#[utoipa::path(
    get,
    path = "/api/v1/admin/stats",
    operation_id = "getPlatformStats",
    tag = crate::ADMIN_TAG,
    responses(
        (status = 200, description = "Platform-wide counts", body = inline(DataEnvelope<PlatformStatsResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    _: AdminStatsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<PlatformStatsResponse>, ApiError> {
    let admin = platform_admin(&identity)?;
    let (user_id, _) = resolve_actor(&state, &identity).await?;

    let stats = state
        .usecase
        .acting_as(user_id)
        .platform_stats(admin)
        .await?;

    Ok(Response::OK(stats.into()))
}

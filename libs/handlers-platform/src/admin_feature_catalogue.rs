use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{AdminFeaturesPath, platform_admin, response::FeatureFlagResponse};

#[utoipa::path(
    get,
    path = "/api/v1/admin/features",
    operation_id = "listFeatureCatalogue",
    tag = crate::ADMIN_TAG,
    responses(
        (status = 200, description = "Every feature flag the running binary knows", body = inline(DataEnvelope<Vec<FeatureFlagResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    _: AdminFeaturesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<FeatureFlagResponse>>, ApiError> {
    let admin = platform_admin(&identity)?;

    let catalogue = state.usecase.list_feature_catalogue(admin);

    Ok(Response::OK(
        catalogue.iter().copied().map(Into::into).collect(),
    ))
}

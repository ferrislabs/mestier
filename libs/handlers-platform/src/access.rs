use auth::Identity;
use axum::Extension;
use handlers::{ApiError, DataEnvelope, Response};

use crate::{AccessPath, platform_admin, response::AdminAccessResponse};

#[utoipa::path(
    get,
    path = "/api/v1/admin/access",
    operation_id = "getAdminAccess",
    tag = crate::ADMIN_TAG,
    responses(
        (status = 200, description = "Whether the caller may use the platform back-office", body = inline(DataEnvelope<AdminAccessResponse>)),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    _: AccessPath,
    Extension(identity): Extension<Identity>,
) -> Result<Response<AdminAccessResponse>, ApiError> {
    let can_access = platform_admin(&identity).is_ok();

    Ok(Response::OK(AdminAccessResponse { can_access }))
}

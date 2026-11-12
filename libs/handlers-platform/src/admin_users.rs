use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
    resolve_actor,
};

use crate::{AdminUsersPath, SearchParams, platform_admin, response::AdminUserResponse};

#[utoipa::path(
    get,
    path = "/api/v1/admin/users",
    operation_id = "listUsersForAdmin",
    tag = crate::ADMIN_TAG,
    params(SearchParams, PaginationParams),
    responses(
        (status = 200, description = "Active users across the platform, newest first, filtered by name, email or username", body = inline(DataEnvelope<Vec<AdminUserResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    _: AdminUsersPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(search): Query<SearchParams>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<AdminUserResponse>, ApiError> {
    let admin = platform_admin(&identity)?;
    let (user_id, _) = resolve_actor(&state, &identity).await?;
    let per_page = pagination.per_page();
    let page = pagination.page();

    let (users, total) = state
        .usecase
        .acting_as(user_id)
        .list_users_for_admin(admin, search.search, per_page, pagination.offset())
        .await?;

    let items: Vec<AdminUserResponse> = users.into_iter().map(Into::into).collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}

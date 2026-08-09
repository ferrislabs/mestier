use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
    resolve_user_id,
};
use mestier_core::application::policy;

use crate::{paths::MembersPath, response::MemberResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/members",
    operation_id = "listMembers",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        PaginationParams,
    ),
    responses(
        (status = 200, description = "Paginated list of members", body = inline(DataEnvelope<Vec<MemberResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, fields(organization_id = %path.organization_id.0, page = pagination.page(), per_page = pagination.per_page()), err)]
pub async fn handler(
    path: MembersPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<MemberResponse>, ApiError> {
    let user_id = resolve_user_id(&state, &identity).await?;
    // TODO: thread JWT realm roles once Identity exposes them.
    let actor = policy::user_subject(user_id, Vec::new());

    let per_page = pagination.per_page();
    let page = pagination.page();
    let offset = pagination.offset();

    // Membership check and per-row account hydration both happen inside
    // `MemberService::list_members` — one query for every occupied seat's
    // account, not one per row.
    let (members, total) = state
        .usecase
        .list_members(actor, path.organization_id, per_page, offset)
        .await?;

    let items: Vec<MemberResponse> = members.into_iter().map(MemberResponse::from).collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}

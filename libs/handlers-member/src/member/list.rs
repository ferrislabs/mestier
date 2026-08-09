use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};
use mestier_core::UserId;

use crate::{
    paths::MembersPath, require_org_membership, resolve_accounts, response::MemberResponse,
};

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
    require_org_membership(&state, &identity, path.organization_id).await?;

    let per_page = pagination.per_page();
    let page = pagination.page();
    let offset = pagination.offset();
    let (members, total) = state
        .usecase
        .list_members(path.organization_id, per_page, offset)
        .await?;

    // One query for every occupied seat's account, not one per row.
    let user_ids: Vec<UserId> = members.iter().filter_map(|member| member.user_id).collect();
    let accounts = resolve_accounts(&state, &user_ids).await?;

    let items: Vec<MemberResponse> = members
        .into_iter()
        .map(|member| {
            let account = member.user_id.and_then(|id| accounts.get(&id).cloned());
            let mut response = MemberResponse::from(member);
            response.account = account;
            response
        })
        .collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}

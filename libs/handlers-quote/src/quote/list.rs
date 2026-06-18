use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};

use crate::{paths::QuotesPath, require_org_membership, response::QuoteResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/quotes",
    operation_id = "listQuotes",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        PaginationParams,
    ),
    responses(
        (status = 200, description = "Paginated list of quotes", body = inline(DataEnvelope<Vec<QuoteResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: QuotesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<QuoteResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let per_page = pagination.per_page();
    let page = pagination.page();
    let offset = pagination.offset();
    let (quotes, total) = state
        .usecase
        .list_quotes(path.organization_id, per_page, offset)
        .await?;
    let items: Vec<QuoteResponse> = quotes.into_iter().map(QuoteResponse::from).collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}

use auth::Identity;
use axum::{Router, middleware::from_fn_with_state};
use axum_extra::routing::RouterExt;
use handlers::{ApiError, AppState, auth::auth_middleware, rate_limit::rate_limit_middleware};
use mestier_core::{CustomerContextId, CustomerId, OrganizationId, Permissions, QuoteId};

pub mod paths;
pub mod pdf;
pub mod quote;
pub mod response;

pub const TAG: &str = "quotes";

#[derive(Debug, serde::Serialize, PartialEq)]
pub struct EmptyResponse;

async fn require_org_membership(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
) -> Result<(), ApiError> {
    let user = state
        .usecase
        .find_user_by_sub(identity.id())
        .await?
        .ok_or(ApiError::Forbidden)?;
    let membership = state
        .usecase
        .find_membership(organization_id, user.id)
        .await?;

    if membership.is_none() {
        return Err(ApiError::Forbidden);
    }

    Ok(())
}

async fn require_quote_membership(
    state: &AppState,
    identity: &Identity,
    quote_id: QuoteId,
) -> Result<mestier_core::Quote, ApiError> {
    let quote = state.usecase.get_quote(quote_id).await?;
    require_org_membership(state, identity, quote.organization_id).await?;

    Ok(quote)
}

/// Membership is the outer gate, `VIEW_QUOTES` is the inner one (#396): a
/// plain member used to be able to read any quote's price under
/// `require_quote_membership` alone, the same gap #395 already closed for
/// customers and invoices. Mirrors `require_view_customers` exactly.
///
/// This guard belongs at the HTTP handler layer only, and must never move
/// into `QuoteService::get_quote`/`list_quotes`: those are also called
/// internally by the write use cases' own `quote.manage` PDP check in
/// `libs/core/src/application/quote/mod.rs`, which has nothing to do with a
/// user browsing a quote.
async fn require_view_quotes(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
) -> Result<(), ApiError> {
    let user = state
        .usecase
        .find_user_by_sub(identity.id())
        .await?
        .ok_or(ApiError::Forbidden)?;

    if state
        .usecase
        .find_membership(organization_id, user.id)
        .await?
        .is_none()
    {
        return Err(ApiError::Forbidden);
    }

    let permissions = state
        .usecase
        .member_permissions(user.id, organization_id)
        .await?;

    if !permissions.contains(Permissions::VIEW_QUOTES) {
        return Err(ApiError::Forbidden);
    }

    Ok(())
}

/// Resolves the quote to its organization, then applies
/// `require_view_quotes` — the read-path counterpart of
/// `require_quote_membership`, returning the loaded `Quote` the same way.
async fn require_quote_view(
    state: &AppState,
    identity: &Identity,
    quote_id: QuoteId,
) -> Result<mestier_core::Quote, ApiError> {
    let quote = state.usecase.get_quote(quote_id).await?;
    require_view_quotes(state, identity, quote.organization_id).await?;

    Ok(quote)
}

async fn require_quote_targets(
    state: &AppState,
    organization_id: OrganizationId,
    customer_id: CustomerId,
    customer_context_id: CustomerContextId,
) -> Result<(), ApiError> {
    let customer = state.usecase.get_customer(customer_id).await?;
    if customer.organization_id != organization_id {
        return Err(ApiError::Forbidden);
    }

    let customer_context = state
        .usecase
        .get_customer_context(customer_context_id)
        .await?;
    if customer_context.customer_id != customer_id {
        return Err(ApiError::Forbidden);
    }

    Ok(())
}

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(quote::list::handler)
        .typed_post(quote::create::handler)
        .typed_get(quote::get_one::handler)
        .typed_patch(quote::update::handler)
        .typed_patch(quote::update_status::handler)
        .typed_delete(quote::soft_delete::handler)
        .typed_get(quote::export_pdf::handler)
        .typed_get(quote::plan_proposal::handler)
        .typed_post(quote::plan::handler)
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}

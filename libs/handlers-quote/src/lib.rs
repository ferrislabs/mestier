use auth::Identity;
use axum::{Router, middleware::from_fn_with_state};
use axum_extra::routing::RouterExt;
use handlers::{ApiError, AppState, auth::auth_middleware, rate_limit::rate_limit_middleware};
use mestier_core::{CustomerId, OrganizationId, PropertyId, QuoteId};

pub mod paths;
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

async fn require_quote_targets(
    state: &AppState,
    organization_id: OrganizationId,
    customer_id: CustomerId,
    property_id: PropertyId,
) -> Result<(), ApiError> {
    let customer = state.usecase.get_customer(customer_id).await?;
    if customer.organization_id != organization_id {
        return Err(ApiError::Forbidden);
    }

    let property = state.usecase.get_property(property_id).await?;
    if property.customer_id != customer_id {
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
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}

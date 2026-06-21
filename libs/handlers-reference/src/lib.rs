use auth::Identity;
use axum::{Router, middleware::from_fn_with_state};
use axum_extra::routing::RouterExt;
use handlers::{ApiError, AppState, auth::auth_middleware, rate_limit::rate_limit_middleware};
use mestier_core::OrganizationId;

pub mod employee;
pub mod equipment;
pub mod paths;
pub mod product;
pub mod response;
pub mod service_rate;

pub const TAG: &str = "reference";

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

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(employee::list::handler)
        .typed_post(employee::create::handler)
        .typed_get(employee::get_one::handler)
        .typed_patch(employee::update::handler)
        .typed_delete(employee::soft_delete::handler)
        .typed_get(equipment::list::handler)
        .typed_post(equipment::create::handler)
        .typed_get(equipment::get_one::handler)
        .typed_patch(equipment::update::handler)
        .typed_delete(equipment::soft_delete::handler)
        .typed_get(service_rate::list::handler)
        .typed_post(service_rate::create::handler)
        .typed_get(service_rate::get_one::handler)
        .typed_patch(service_rate::update::handler)
        .typed_delete(service_rate::soft_delete::handler)
        .typed_get(product::list::handler)
        .typed_post(product::create::handler)
        .typed_get(product::get_one::handler)
        .typed_patch(product::update::handler)
        .typed_delete(product::soft_delete::handler)
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}

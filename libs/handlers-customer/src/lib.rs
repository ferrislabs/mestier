use auth::Identity;
use axum::{Router, middleware::from_fn_with_state};
use axum_extra::routing::RouterExt;
use handlers::{ApiError, AppState, auth::auth_middleware, rate_limit::rate_limit_middleware};
use mestier_core::{CustomerContactId, CustomerId, OrganizationId, Permissions};

pub mod customer;
pub mod customer_contact;
pub mod customer_context;
pub mod paths;
pub mod response;

pub const TAG: &str = "customers";

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

async fn require_customer_membership(
    state: &AppState,
    identity: &Identity,
    customer_id: CustomerId,
) -> Result<(), ApiError> {
    let customer = state.usecase.get_customer(customer_id).await?;
    require_org_membership(state, identity, customer.organization_id).await
}

async fn require_customer_contact_membership(
    state: &AppState,
    identity: &Identity,
    customer_contact_id: CustomerContactId,
) -> Result<(), ApiError> {
    let customer_contact = state
        .usecase
        .get_customer_contact(customer_contact_id)
        .await?;
    require_customer_membership(state, identity, customer_contact.customer_id).await
}

/// Membership is the outer gate, `VIEW_CUSTOMERS` is the inner one (#395):
/// belonging to the organization used to be enough to browse its customers
/// and their contacts/contexts, the same gap #305/#306 already closed for
/// writes and for reports. Mirrors `require_view_reports` in
/// `handlers-reporting` exactly.
///
/// This guard belongs at the HTTP handler layer only. It must never move
/// into `CustomerService::get_customer`/`list_customers`: those are also
/// called internally to validate that a customer id referenced by another
/// entity (e.g. an invoice) belongs to the organization, a check that has
/// nothing to do with a user browsing a customer — see the note on
/// `invoice.manage` in `libs/core/src/application/mod.rs`.
async fn require_view_customers(
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

    if !permissions.contains(Permissions::VIEW_CUSTOMERS) {
        return Err(ApiError::Forbidden);
    }

    Ok(())
}

/// Resolves the customer to its organization, then applies
/// `require_view_customers` — the read-path counterpart of
/// `require_customer_membership`, used by the contact/context list and
/// detail handlers that only carry a `customer_id`.
async fn require_customer_view(
    state: &AppState,
    identity: &Identity,
    customer_id: CustomerId,
) -> Result<(), ApiError> {
    let customer = state.usecase.get_customer(customer_id).await?;
    require_view_customers(state, identity, customer.organization_id).await
}

/// Resolves the contact to its customer, then applies
/// `require_customer_view` — the read-path counterpart of
/// `require_customer_contact_membership`.
async fn require_customer_contact_view(
    state: &AppState,
    identity: &Identity,
    customer_contact_id: CustomerContactId,
) -> Result<(), ApiError> {
    let customer_contact = state
        .usecase
        .get_customer_contact(customer_contact_id)
        .await?;
    require_customer_view(state, identity, customer_contact.customer_id).await
}

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(customer::list::handler)
        .typed_post(customer::create::handler)
        .typed_get(customer::get_one::handler)
        .typed_patch(customer::update::handler)
        .typed_delete(customer::soft_delete::handler)
        .typed_get(customer_contact::list::handler)
        .typed_post(customer_contact::create::handler)
        .typed_get(customer_contact::get_one::handler)
        .typed_patch(customer_contact::update::handler)
        .typed_delete(customer_contact::soft_delete::handler)
        .typed_get(customer_context::list::handler)
        .typed_post(customer_context::create::handler)
        .typed_get(customer_context::get_one::handler)
        .typed_patch(customer_context::update::handler)
        .typed_delete(customer_context::soft_delete::handler)
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}

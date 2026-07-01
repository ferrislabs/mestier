use auth::Identity;
use axum::{Router, middleware::from_fn_with_state};
use axum_extra::routing::RouterExt;
use handlers::{ApiError, AppState, auth::auth_middleware, rate_limit::rate_limit_middleware};
use mestier_core::{CustomerContextId, CustomerId, Invoice, InvoiceId, OrganizationId};

pub mod invoice;
pub mod paths;
pub mod response;

pub const TAG: &str = "invoices";

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

async fn require_invoice_membership(
	state: &AppState,
	identity: &Identity,
	invoice_id: InvoiceId,
) -> Result<Invoice, ApiError> {
	let invoice = state.usecase.get_invoice(invoice_id).await?;
	require_org_membership(state, identity, invoice.org_id).await?;

	Ok(invoice)
}

async fn require_invoice_targets(
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
		.typed_get(invoice::list::handler)
		.typed_post(invoice::create::handler)
		.typed_get(invoice::get_one::handler)
		.typed_patch(invoice::update::handler)
		.typed_patch(invoice::update_status::handler)
		.typed_delete(invoice::soft_delete::handler)
		.typed_post(invoice::convert_from_quote::handler)
		.typed_post(invoice::create_deposit::handler)
		.typed_post(invoice::create_balance::handler)
		.layer(from_fn_with_state(state.clone(), rate_limit_middleware))
		.layer(from_fn_with_state(state.clone(), auth_middleware))
}

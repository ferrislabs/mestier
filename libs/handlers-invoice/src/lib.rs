//! The invoice API (#319): routes, request/response DTOs and the PDF export
//! for the invoice aggregate #316-#320 built in the domain layer.
//!
//! A new crate, not a module folded into `handlers-quote`: a quote and an
//! invoice are the same commercial area but two genuinely different
//! aggregates — different mutability rule (a quote is edited until
//! accepted, an invoice is immutable from the moment it is issued),
//! different numbering, different kinds (`STANDARD`/`DEPOSIT`/`FINAL`/
//! `CREDIT_NOTE` have no quote equivalent). Folding the two into one crate
//! would blur exactly the boundary #316 drew on purpose. The two documents
//! do share one thing genuinely worth sharing — the low-level PDF/text
//! machinery, issuer block, VAT breakdown and legal mentions must render
//! identically on both — which is why this crate depends on
//! `handlers-quote` for `handlers_quote::pdf`, rather than the reverse or a
//! third shared crate: `handlers-quote` already owned that renderer, and a
//! third crate for one module would be a layer with nothing else in it.

use auth::Identity;
use axum::{Router, middleware::from_fn_with_state};
use axum_extra::routing::RouterExt;
use handlers::{ApiError, AppState, auth::auth_middleware, rate_limit::rate_limit_middleware};
use mestier_core::{
    CustomerContextId, CustomerId, Invoice, InvoicePayment, InvoicePaymentId, OrganizationId,
    Project, ProjectId,
};

pub mod invoice;
pub mod paths;
pub mod payment;
pub mod project;
pub mod response;

pub const TAG: &str = "invoices";

#[derive(Debug, serde::Serialize, PartialEq)]
pub struct EmptyResponse;

/// Each handler crate defines its own copy of this check rather than
/// sharing one (confirmed against `handlers-quote` and `handlers-planning`,
/// which both do the same) — small enough that a shared crate for it would
/// be more indirection than the duplication it removes.
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

/// Loads the invoice, then checks membership against the loaded row's own
/// `organization_id` — never against anything in the path, since every
/// bare-`invoice_id` route (CLAUDE.md: "bare ids derive their organization
/// from the loaded row, never from the path") carries no organization
/// segment at all.
async fn require_invoice_membership(
    state: &AppState,
    identity: &Identity,
    invoice_id: mestier_core::InvoiceId,
) -> Result<Invoice, ApiError> {
    let invoice = state.usecase.get_invoice(invoice_id).await?;
    require_org_membership(state, identity, invoice.organization_id).await?;

    Ok(invoice)
}

/// Same shape as `require_invoice_membership`, for the project-scoped
/// routes (`issue_deposit`/`issue_final_invoice`/`billing-summary`/the
/// project-scoped invoice listing) — mirrors `require_project` in
/// `handlers-planning/src/project/mod.rs`, minus the extra
/// `organization_id`-from-path check that function does: these routes carry
/// no organization segment, so the project's own row is the only source of
/// truth for it.
async fn require_project_membership(
    state: &AppState,
    identity: &Identity,
    project_id: ProjectId,
) -> Result<Project, ApiError> {
    let project = state.usecase.get_project(project_id).await?;
    require_org_membership(state, identity, project.organization_id).await?;

    Ok(project)
}

/// Resolves a payment on its own — the delete-by-payment-id route has
/// nothing else to start from. `InvoicePayment` already carries its own
/// `organization_id`, so this needs no second `get_invoice` call the way
/// loading the containing invoice first would.
async fn require_payment_membership(
    state: &AppState,
    identity: &Identity,
    invoice_payment_id: InvoicePaymentId,
) -> Result<InvoicePayment, ApiError> {
    let payment = state
        .usecase
        .find_payment_by_id(invoice_payment_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    require_org_membership(state, identity, payment.organization_id).await?;

    Ok(payment)
}

/// Validates that a customer and its context belong to `organization_id`
/// and to each other — mirrors `require_quote_targets` in `handlers-quote`.
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

/// When an invoice names a project, the project must belong to the same
/// organization — same "a real id from another organization must not be
/// usable" rule `require_invoice_targets` enforces for the customer.
async fn require_invoice_project(
    state: &AppState,
    organization_id: OrganizationId,
    project_id: ProjectId,
) -> Result<(), ApiError> {
    let project = state.usecase.get_project(project_id).await?;
    if project.organization_id != organization_id {
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
        .typed_post(invoice::issue::handler)
        .typed_post(invoice::cancel::handler)
        .typed_post(invoice::credit_note::handler)
        .typed_get(invoice::export_pdf::handler)
        .typed_get(payment::list::handler)
        .typed_post(payment::record::handler)
        .typed_delete(payment::delete::handler)
        .typed_get(project::list::handler)
        .typed_get(project::billing_summary::handler)
        .typed_post(project::deposit::handler)
        .typed_post(project::final_invoice::handler)
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}

//! The purchasing API (#339): receiving somebody else's invoice, not
//! issuing our own.
//!
//! A new crate, not folded into `handlers-invoice`: an issued invoice and a
//! received one share almost no rules — different mutability shape
//! (`invoices` is immutable from the moment it leaves `Draft`;
//! `supplier_invoices` is immutable from the instant it exists, only our
//! own review metadata ever changes), no numbering on our side, no VAT
//! computation on our side, a document that must be storable before its
//! merchant is even recognised. They will often be read together at the
//! API-consumer level (an accounts-payable view wants both), but that is
//! what the router merge and the OpenAPI tag solve, not what a shared
//! crate would.

use auth::Identity;
use axum::{Router, middleware::from_fn_with_state};
use axum_extra::routing::RouterExt;
use handlers::{ApiError, AppState, auth::auth_middleware, rate_limit::rate_limit_middleware};
use mestier_core::{OrganizationId, Project, ProjectId, SupplierInvoice, SupplierInvoiceLine};

pub mod allocation;
pub mod paths;
pub mod response;
pub mod supplier_invoice;

pub const TAG: &str = "purchasing";

/// Each handler crate defines its own copy of this check rather than
/// sharing one — see `handlers-invoice`'s identical comment.
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
/// bare-`supplier_invoice_id` route carries no organization segment at all
/// (CLAUDE.md: "bare ids derive their organization from the loaded row").
async fn require_supplier_invoice_membership(
    state: &AppState,
    identity: &Identity,
    supplier_invoice_id: mestier_core::SupplierInvoiceId,
) -> Result<SupplierInvoice, ApiError> {
    let invoice = state
        .usecase
        .get_supplier_invoice(supplier_invoice_id)
        .await?;
    require_org_membership(state, identity, invoice.organization_id).await?;

    Ok(invoice)
}

/// Same shape as [`require_supplier_invoice_membership`], for the
/// bare-`supplier_invoice_line_id` allocation route.
async fn require_supplier_invoice_line_membership(
    state: &AppState,
    identity: &Identity,
    supplier_invoice_line_id: mestier_core::SupplierInvoiceLineId,
) -> Result<SupplierInvoiceLine, ApiError> {
    let line = state
        .usecase
        .find_supplier_invoice_line(supplier_invoice_line_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    require_org_membership(state, identity, line.organization_id).await?;

    Ok(line)
}

/// Same shape again, for the project-scoped supplier-costs read — mirrors
/// `require_project_membership` in `handlers-invoice`.
async fn require_project_membership(
    state: &AppState,
    identity: &Identity,
    project_id: ProjectId,
) -> Result<Project, ApiError> {
    let project = state.usecase.get_project(project_id).await?;
    require_org_membership(state, identity, project.organization_id).await?;

    Ok(project)
}

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_post(supplier_invoice::import::handler)
        .typed_get(supplier_invoice::list::handler)
        .typed_get(supplier_invoice::get_one::handler)
        .typed_patch(supplier_invoice::update::handler)
        .typed_post(supplier_invoice::confirm::handler)
        .typed_post(supplier_invoice::reject::handler)
        .typed_put(allocation::replace::handler)
        .typed_get(allocation::project_costs::handler)
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}

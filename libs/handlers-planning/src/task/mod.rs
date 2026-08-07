//! Work orders ("chantiers"): CRUD plus the transactional `PATCH` that
//! reschedules and reassigns in one call.
//!
//! Owns its own leaf paths (see the crate-level `paths.rs` docstring) since
//! `organization_id` and `work_order_id` are both part of every single-item
//! route, unlike the flatter `/things/{id}` shape used elsewhere in the repo.

use auth::Identity;
use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use handlers::{ApiError, AppState};
use mestier_core::{CustomerContextId, CustomerId, QuoteId, WorkOrder, WorkOrderId};
use serde::Deserialize;

use crate::require_org_membership;

pub mod create;
pub mod get_one;
pub mod list;
pub mod soft_delete;
pub mod update;

/// This aggregate's routes, unlayered — the crate-level `router` in
/// `lib.rs` merges every aggregate submodule's router before applying the
/// shared rate-limit/auth middleware once, rather than each submodule
/// layering its own.
pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_post(create::handler)
        .typed_get(get_one::handler)
        .typed_patch(update::handler)
        .typed_delete(soft_delete::handler)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/work-orders")]
pub struct WorkOrdersPath {
    pub organization_id: mestier_core::OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/work-orders/{work_order_id}")]
pub struct WorkOrderPath {
    pub organization_id: mestier_core::OrganizationId,
    pub work_order_id: WorkOrderId,
}

/// Loads the work order and checks both that the caller belongs to
/// `organization_id` and that the work order actually belongs to it —
/// `organization_id` is part of every route, so a mismatch (a real
/// `work_order_id` from a different organization) is treated the same as
/// "does not exist" rather than leaking cross-tenant existence via 403.
pub(crate) async fn require_work_order(
    state: &AppState,
    identity: &Identity,
    organization_id: mestier_core::OrganizationId,
    work_order_id: WorkOrderId,
) -> Result<WorkOrder, ApiError> {
    require_org_membership(state, identity, organization_id).await?;

    let work_order = state.usecase.get_work_order(work_order_id).await?;
    if work_order.organization_id != organization_id {
        return Err(ApiError::NotFound);
    }

    Ok(work_order)
}

/// Validates that `customer_id`/`customer_context_id` (and `quote_id`, when
/// present) actually belong to `organization_id` — and to each other —
/// before a work order is created or its targets change. Mirrors
/// `handlers-quote`'s `require_quote_targets`.
pub(crate) async fn require_work_order_targets(
    state: &AppState,
    organization_id: mestier_core::OrganizationId,
    customer_id: CustomerId,
    customer_context_id: CustomerContextId,
    quote_id: Option<QuoteId>,
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

    if let Some(quote_id) = quote_id {
        let quote = state.usecase.get_quote(quote_id).await?;
        if quote.organization_id != organization_id {
            return Err(ApiError::Forbidden);
        }
    }

    Ok(())
}

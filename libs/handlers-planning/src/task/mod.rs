//! Tasks — the planning module's unit, of which a "chantier" is simply the
//! case that carries a customer. CRUD plus the transactional `PATCH` that
//! reschedules and reassigns in one call.
//!
//! Owns its own leaf paths (see the crate-level `paths.rs` docstring) since
//! `organization_id` and `task_id` are both part of every single-item
//! route, unlike the flatter `/things/{id}` shape used elsewhere in the repo.

use auth::Identity;
use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use handlers::{ApiError, AppState};
use mestier_core::{CustomerContextId, CustomerId, QuoteId, Task, TaskId};
use serde::Deserialize;

use crate::require_org_membership;

pub mod bulk_assign;
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
        .typed_post(bulk_assign::handler)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/tasks")]
pub struct TasksPath {
    pub organization_id: mestier_core::OrganizationId,
}

/// A distinct literal path rather than a query/body flag on `TasksPath`'s
/// own `POST` (`create`): a bulk operation has a different shape end to
/// end (a list of ids in, a list of tasks out) and a different failure
/// contract (all-or-nothing across many tasks, not one task's validation).
/// Static segments win over `{task_id}` in axum's router, so this never
/// collides with `TaskPath`.
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/tasks/bulk-assign")]
pub struct TasksBulkAssignPath {
    pub organization_id: mestier_core::OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/tasks/{task_id}")]
pub struct TaskPath {
    pub organization_id: mestier_core::OrganizationId,
    pub task_id: TaskId,
}

/// Loads the task and checks both that the caller belongs to
/// `organization_id` and that the task actually belongs to it —
/// `organization_id` is part of every route, so a mismatch (a real
/// `task_id` from a different organization) is treated the same as "does
/// not exist" rather than leaking cross-tenant existence via 403.
pub(crate) async fn require_task(
    state: &AppState,
    identity: &Identity,
    organization_id: mestier_core::OrganizationId,
    task_id: TaskId,
) -> Result<Task, ApiError> {
    require_org_membership(state, identity, organization_id).await?;

    let task = state.usecase.get_task(task_id).await?;
    if task.organization_id != organization_id {
        return Err(ApiError::NotFound);
    }

    Ok(task)
}

/// Validates that, when present, `customer_id`/`customer_context_id` (and
/// `quote_id`) actually belong to `organization_id` — and to each other —
/// before a task is created. A task with neither `customer_id` nor
/// `customer_context_id` is simply not a chantier, and nothing is checked.
/// Mirrors `handlers-quote`'s `require_quote_targets`.
pub(crate) async fn require_task_targets(
    state: &AppState,
    organization_id: mestier_core::OrganizationId,
    customer_id: Option<CustomerId>,
    customer_context_id: Option<CustomerContextId>,
    quote_id: Option<QuoteId>,
) -> Result<(), ApiError> {
    if let (Some(customer_id), Some(customer_context_id)) = (customer_id, customer_context_id) {
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
    }

    if let Some(quote_id) = quote_id {
        let quote = state.usecase.get_quote(quote_id).await?;
        if quote.organization_id != organization_id {
            return Err(ApiError::Forbidden);
        }
    }

    Ok(())
}

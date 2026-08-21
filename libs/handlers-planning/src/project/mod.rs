//! Projects — the subject a cost is reported against.
//!
//! A project exists with or without a customer, which is the whole point: a
//! recurring meeting bills nobody and still costs money. See
//! `docs/adr/0002-planned-cost-model.md`.
//!
//! There is no route here for attaching a task. That happens on
//! `PATCH /tasks/{task_id}` through `project_id`, so there is exactly one way
//! to express the link.
//!
//! Owns its own leaf paths, like `task` does and for the same reason:
//! `organization_id` and `project_id` are both part of every single-item route.

use auth::Identity;
use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use handlers::{ApiError, AppState};
use mestier_core::{CustomerContextId, CustomerId, Project, ProjectId, QuoteId};
use serde::Deserialize;

use crate::require_org_membership;

pub mod archive;
pub mod create;
pub mod get_one;
pub mod list;
pub mod restore;
pub mod update;

/// This aggregate's routes, unlayered — the crate-level `router` in `lib.rs`
/// merges every aggregate submodule's router before applying the shared
/// rate-limit/auth middleware once.
pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_post(create::handler)
        .typed_get(get_one::handler)
        .typed_patch(update::handler)
        .typed_delete(archive::handler)
        .typed_post(restore::handler)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/projects")]
pub struct ProjectsPath {
    pub organization_id: mestier_core::OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/projects/{project_id}")]
pub struct ProjectPath {
    pub organization_id: mestier_core::OrganizationId,
    pub project_id: ProjectId,
}

/// Un-archiving is its own verb rather than a `PATCH` field: the update route
/// replaces the project's descriptive fields, and folding a lifecycle
/// transition into it would let a rename silently revive a retired project.
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/projects/{project_id}/restore")]
pub struct ProjectRestorePath {
    pub organization_id: mestier_core::OrganizationId,
    pub project_id: ProjectId,
}

/// Loads the project and checks both that the caller belongs to
/// `organization_id` and that the project actually belongs to it. A real id
/// from another organization is treated exactly like one that does not exist,
/// rather than leaking cross-tenant existence through a 403.
pub(crate) async fn require_project(
    state: &AppState,
    identity: &Identity,
    organization_id: mestier_core::OrganizationId,
    project_id: ProjectId,
) -> Result<Project, ApiError> {
    require_org_membership(state, identity, organization_id).await?;

    let project = state.usecase.get_project(project_id).await?;
    if project.organization_id != organization_id {
        return Err(ApiError::NotFound);
    }

    Ok(project)
}

/// Validates that a customer, its context and a quote all belong to
/// `organization_id` and to each other before they land on a project.
///
/// Unlike `require_task_targets`, a customer with no context is accepted: the
/// domain relaxed that pairing, and a project for a client whose site is not
/// pinned down yet is a real case.
pub(crate) async fn require_project_targets(
    state: &AppState,
    organization_id: mestier_core::OrganizationId,
    customer_id: Option<CustomerId>,
    customer_context_id: Option<CustomerContextId>,
    quote_id: Option<QuoteId>,
) -> Result<(), ApiError> {
    if let Some(customer_id) = customer_id {
        let customer = state.usecase.get_customer(customer_id).await?;
        if customer.organization_id != organization_id {
            return Err(ApiError::Forbidden);
        }

        if let Some(customer_context_id) = customer_context_id {
            let customer_context = state
                .usecase
                .get_customer_context(customer_context_id)
                .await?;
            if customer_context.customer_id != customer_id {
                return Err(ApiError::Forbidden);
            }
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

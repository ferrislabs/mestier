//! Workflows: list, create, read (with its current version), save a new
//! version, enable/disable and rename (`update`), delete.

use auth::Identity;
use axum::Router;
use axum_extra::routing::RouterExt;
use handlers::{ApiError, AppState};
use mestier_core::{OrganizationId, Workflow};
use uuid::Uuid;

use crate::require_org_membership;

pub mod create;
pub mod delete;
pub mod get_one;
pub mod list;
pub mod save_version;
pub mod update;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_post(create::handler)
        .typed_get(get_one::handler)
        .typed_patch(update::handler)
        .typed_delete(delete::handler)
        .typed_put(save_version::handler)
}

/// Loads the workflow and checks both that the caller belongs to
/// `organization_id` and that the workflow actually belongs to it — mirrors
/// `credential::require_credential` and `handlers-planning::task::require_task`.
pub(crate) async fn require_workflow(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
    workflow_id: Uuid,
) -> Result<Workflow, ApiError> {
    require_org_membership(state, identity, organization_id).await?;

    state
        .usecase
        .find_workflow(organization_id, workflow_id)
        .await?
        .ok_or(ApiError::NotFound)
}

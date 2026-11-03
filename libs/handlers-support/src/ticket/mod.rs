use std::str::FromStr;

use auth::Identity;
use authz::Subject;
use axum::Router;
use axum_extra::routing::RouterExt;
use handlers::{ApiError, AppState, resolve_actor};
use mestier_core::{OrganizationId, TicketStatus};

use crate::require_org_membership;

pub mod add_badge;
pub mod assign;
pub mod get_one;
pub mod list;
pub mod remove_badge;
pub mod transition;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_get(get_one::handler)
        .typed_patch(transition::handler)
        .typed_put(assign::handler)
        .typed_post(add_badge::handler)
        .typed_delete(remove_badge::handler)
}

pub(crate) async fn require_manager(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
) -> Result<Subject, ApiError> {
    require_org_membership(state, identity, organization_id).await?;
    let (_, actor) = resolve_actor(state, identity).await?;

    Ok(actor)
}

pub(crate) fn parse_status(value: &str) -> Result<TicketStatus, ApiError> {
    TicketStatus::from_str(value)
        .map_err(|_| ApiError::UnprocessableEntity(format!("unknown ticket status `{value}`")))
}

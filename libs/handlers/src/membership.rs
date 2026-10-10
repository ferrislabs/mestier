use auth::Identity;
use common::OrganizationId;

use crate::{ApiError, AppState};

pub const MCP_DISABLED_MESSAGE: &str =
    "L'accès par IA (MCP) est désactivé pour cette organisation.";

pub async fn require_mcp_access(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
) -> Result<(), ApiError> {
    if !identity.is_delegated() {
        return Ok(());
    }

    let organization = state.usecase.get_organization(organization_id).await?;
    if organization.mcp_enabled {
        Ok(())
    } else {
        Err(ApiError::ForbiddenWithReason(
            MCP_DISABLED_MESSAGE.to_owned(),
        ))
    }
}

pub async fn require_org_membership(
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

    require_mcp_access(state, identity, organization_id).await
}

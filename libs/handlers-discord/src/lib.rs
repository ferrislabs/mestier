use auth::Identity;
use axum::{Router, middleware::from_fn_with_state};
use axum_extra::routing::RouterExt;
use common::OrganizationId;
use discord::ChannelId;
use handlers::{ApiError, AppState, auth::auth_middleware, rate_limit::rate_limit_middleware};
use mestier_core::Permissions;

pub mod category;
pub mod channel;
pub mod gateway;
pub mod message;
pub mod notification;
pub mod paths;
pub mod presence;
pub mod reaction;
pub mod read_state;
pub mod response;
pub mod thread;
pub mod typing;
pub mod webhook;

pub const TAG: &str = "discord";

#[derive(Debug, serde::Serialize, PartialEq)]
pub struct EmptyResponse;

/// Verify the caller is a member of the given org.
/// Returns `ApiError::Forbidden` when the user has no membership record.
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

    Ok(())
}

/// Verify membership AND that the member holds the required permission bit.
///
/// Delegates entirely to `MestierUseCase::authorize_action`, which uses the
/// canonical `LocalPolicyEngine` (`self.authz`) configured by
/// `mestier_core::application::default_authorizer`.  The action → bit map
/// lives in exactly one place (core); this helper only translates the
/// `CoreError::Forbidden` result to `ApiError`.
pub async fn require_permission(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
    action: &str,
) -> Result<(), ApiError> {
    let user = state
        .usecase
        .find_user_by_sub(identity.id())
        .await?
        .ok_or(ApiError::Forbidden)?;

    // TODO: thread JWT realm roles once Identity exposes them.
    let iam_roles: Vec<String> = Vec::new();

    state
        .usecase
        .authorize_action(user.id, iam_roles, organization_id, action)
        .await?;

    Ok(())
}

/// Resolve the caller's effective channel-scoped bits (org-role bits adjusted by
/// per-channel overwrites) and return `ApiError::Forbidden` when the given `bit`
/// is absent.
///
/// This replaces org-level `require_permission` for channel-scoped paths so that
/// per-channel role/member overwrites can grant or deny access independently of
/// org-role assignments.
pub async fn require_channel_permission(
    state: &AppState,
    identity: &Identity,
    channel_id: ChannelId,
    bit: Permissions,
) -> Result<(), ApiError> {
    let user = state
        .usecase
        .find_user_by_sub(identity.id())
        .await?
        .ok_or(ApiError::Forbidden)?;

    let effective = state
        .usecase
        .resolve_channel_permissions(user.id, channel_id)
        .await?;

    if !effective.contains(bit) {
        return Err(ApiError::Forbidden);
    }

    Ok(())
}

pub fn router(state: &AppState) -> Router<AppState> {
    // Authenticated routes: FerrisKey OIDC (`auth_middleware`) + rate-limit.
    let authed = Router::new()
        .typed_get(category::list::handler)
        .typed_post(category::create::handler)
        .typed_patch(category::update::handler)
        .typed_delete(category::delete::handler)
        .typed_get(channel::list::handler)
        .typed_post(channel::create::handler)
        .typed_get(channel::get_one::handler)
        .typed_patch(channel::update::handler)
        .typed_delete(channel::delete::handler)
        .typed_get(thread::list::handler)
        .typed_post(thread::create::handler)
        .typed_patch(thread::update::handler)
        .typed_delete(thread::delete::handler)
        .typed_get(message::list::handler)
        .typed_post(message::create::handler)
        .typed_patch(message::update::handler)
        .typed_delete(message::delete::handler)
        .typed_put(reaction::add::handler)
        .typed_delete(reaction::remove::handler)
        .typed_get(reaction::list::handler)
        .typed_get(webhook::list::handler)
        .typed_post(webhook::create::handler)
        .typed_patch(webhook::update::handler)
        .typed_delete(webhook::delete::handler)
        .typed_put(presence::set::handler)
        .typed_post(typing::start::handler)
        .typed_put(read_state::mark::handler)
        .typed_get(read_state::unread::handler)
        .typed_get(notification::list::handler)
        .typed_put(notification::mark_read::handler)
        .typed_put(notification::mark_all_read::handler)
        .typed_get(channel::permissions::list::handler)
        .typed_put(channel::permissions::upsert_everyone::handler)
        .typed_put(channel::permissions::upsert_target::handler)
        .typed_delete(channel::permissions::delete_everyone::handler)
        .typed_delete(channel::permissions::delete_target::handler)
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware));

    // Public routes that authenticate THEMSELVES — they must NOT go behind
    // `auth_middleware`: Task 8 mounts `webhook::execute` (webhook-token auth)
    // and Task 10 mounts `gateway` (WS `identify` handshake) here.
    let public = Router::new()
        .typed_post(webhook::execute::handler)
        .typed_get(gateway::handler)
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware));

    Router::new().merge(authed).merge(public)
}

#[cfg(test)]
mod tests {
    use mestier_core::Permissions;

    #[test]
    fn permissions_view_channel_bit_is_32() {
        assert_eq!(Permissions::VIEW_CHANNEL.bits(), 32);
    }

    #[test]
    fn permissions_send_messages_bit_is_64() {
        assert_eq!(Permissions::SEND_MESSAGES.bits(), 64);
    }

    #[test]
    fn permissions_manage_channels_bit_is_8() {
        assert_eq!(Permissions::MANAGE_CHANNELS.bits(), 8);
    }

    #[test]
    fn permissions_contains_reflects_channel_bits() {
        let effective = Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES;
        assert!(effective.contains(Permissions::VIEW_CHANNEL));
        assert!(effective.contains(Permissions::SEND_MESSAGES));
        assert!(!effective.contains(Permissions::MANAGE_CHANNELS));
    }
}

use std::collections::HashMap;

use auth::Identity;
use axum::{Router, middleware::from_fn_with_state};
use axum_extra::routing::RouterExt;
use handlers::{ApiError, AppState, auth::auth_middleware, rate_limit::rate_limit_middleware};
use mestier_core::{OrganizationId, UserId};

use crate::response::MemberAccountResponse;

pub mod member;
pub mod paths;
pub mod response;

pub const TAG: &str = "members";

#[derive(Debug, serde::Serialize, PartialEq)]
pub struct EmptyResponse;

/// Verify the caller is a member of the given org. Copied from
/// `libs/handlers-reference/src/lib.rs` — read-only routes (`list`, `get`)
/// need only membership, not `MANAGE_MEMBERS`.
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

/// Verify membership AND that the member holds the required permission bit.
/// Copied from `libs/handlers-discord/src/lib.rs` — delegates entirely to
/// `MestierUseCase::authorize_action`, the PDP entry point; a handler never
/// builds a `Subject` itself.
async fn require_permission(
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

/// Resolves the account behind a single occupied seat. `None` when the seat
/// is vacant (`user_id: None`) — no query needed — or when the linked user
/// row cannot be found (never surfaced as an error: a member response degrades
/// to a vacant-looking seat rather than failing the whole request).
pub(crate) async fn resolve_account(
    state: &AppState,
    user_id: Option<UserId>,
) -> Result<Option<MemberAccountResponse>, ApiError> {
    let Some(user_id) = user_id else {
        return Ok(None);
    };

    let user = state.usecase.find_user_by_id(user_id).await?;

    Ok(user.map(|user| MemberAccountResponse {
        email: user.email,
        name: user.name,
    }))
}

/// Resolves every account behind a page of seats in one query — the
/// list-route counterpart of [`resolve_account`], keyed by `UserId` so the
/// caller can attach each member's account without a per-row lookup.
pub(crate) async fn resolve_accounts(
    state: &AppState,
    user_ids: &[UserId],
) -> Result<HashMap<UserId, MemberAccountResponse>, ApiError> {
    if user_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let users = state.usecase.list_users_by_ids(user_ids).await?;

    Ok(users
        .into_iter()
        .map(|user| {
            (
                user.id,
                MemberAccountResponse {
                    email: user.email,
                    name: user.name,
                },
            )
        })
        .collect())
}

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(member::list::handler)
        .typed_post(member::create::handler)
        .typed_get(member::get_one::handler)
        .typed_patch(member::update::handler)
        .typed_delete(member::soft_delete::handler)
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}

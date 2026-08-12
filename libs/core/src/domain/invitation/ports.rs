use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{
    UserId,
    domain::{
        invitation::{Invitation, InvitationId},
        organization::OrganizationId,
    },
};

#[cfg_attr(test, mockall::automock)]
pub trait InvitationRepository: Send {
    fn insert(
        &mut self,
        invitation: &Invitation,
    ) -> impl Future<Output = Result<Invitation, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        invitation_id: InvitationId,
    ) -> impl Future<Output = Result<Option<Invitation>, CoreError>> + Send;

    /// Exact-match lookup on the hashed token — see `token::hash`. The
    /// caller hashes the presented clear token before calling this; the
    /// repository never sees a clear value.
    fn find_by_token_hash(
        &mut self,
        token_hash: &[u8],
    ) -> impl Future<Output = Result<Option<Invitation>, CoreError>> + Send;

    /// Pending only (`consumed_at IS NULL`) — what `GET .../invitations`
    /// shows the admin.
    fn list_pending_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<Invitation>, CoreError>> + Send;

    /// Marks the invitation accepted. The row is never deleted here — the
    /// point of `consumed_at` over a hard delete is that it stays auditable.
    fn mark_consumed(
        &mut self,
        invitation_id: InvitationId,
        consumed_at: DateTime<Utc>,
        consumed_by_user_id: UserId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Deletes a *pending* invitation. Unlike acceptance, revoking a link
    /// that was never used has nothing worth auditing, so a hard delete
    /// (guarded to pending rows only) is enough. `CoreError::NotFound` when
    /// no pending row matches `invitation_id` — gone, or already consumed.
    fn revoke(
        &mut self,
        invitation_id: InvitationId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

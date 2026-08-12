use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    UserId,
    domain::{member::MemberId, organization::OrganizationId},
};

pub mod commands;
pub mod events;
pub mod ports;
pub mod service;
pub mod token;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct InvitationId(pub Uuid);

impl FromStr for InvitationId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(InvitationId)
    }
}

impl Display for InvitationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A shareable, single-use link that grants access to an organization.
///
/// `token_hash` is the only trace of the token this type ever carries — the
/// clear value lives nowhere past the moment [`token::generate`] returns it;
/// see `InvitationService::invite_member`.
#[derive(Debug, Clone)]
pub struct Invitation {
    pub id: InvitationId,
    pub organization_id: OrganizationId,
    /// `Some` — grants login access to a seat #181's member API already
    /// created. `None` — acceptance creates the seat itself, named from the
    /// FerrisKey account. See `InvitationService::accept_invitation`.
    pub member_id: Option<MemberId>,
    pub token_hash: Vec<u8>,
    pub expires_at: DateTime<Utc>,
    /// `None` while pending. Set exactly once, by `accept_invitation` — the
    /// token is consumed, never deleted, so the row stays auditable.
    pub consumed_at: Option<DateTime<Utc>>,
    pub consumed_by_user_id: Option<UserId>,
    pub created_by_user_id: UserId,
    pub created_at: DateTime<Utc>,
}

impl Invitation {
    pub fn is_pending(&self) -> bool {
        self.consumed_at.is_none()
    }

    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        self.expires_at <= now
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn invitation(consumed_at: Option<DateTime<Utc>>, expires_at: DateTime<Utc>) -> Invitation {
        Invitation {
            id: InvitationId(Uuid::new_v4()),
            organization_id: OrganizationId(Uuid::new_v4()),
            member_id: None,
            token_hash: vec![0u8; 32],
            expires_at,
            consumed_at,
            consumed_by_user_id: None,
            created_by_user_id: UserId(Uuid::new_v4()),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn invitation_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = InvitationId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn is_pending_is_true_until_consumed() {
        let now = Utc::now();
        assert!(invitation(None, now + chrono::Duration::days(1)).is_pending());
        assert!(!invitation(Some(now), now + chrono::Duration::days(1)).is_pending());
    }

    #[test]
    fn is_expired_compares_against_the_given_instant() {
        let now = Utc::now();
        let expiring = invitation(None, now + chrono::Duration::minutes(1));

        assert!(!expiring.is_expired(now));
        assert!(expiring.is_expired(now + chrono::Duration::minutes(2)));
    }
}

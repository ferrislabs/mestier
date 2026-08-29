//! Authorization helpers shared by every domain service.
//!
//! Two responsibilities:
//!
//! 1. **Subject conventions** — keys under which Mestier stores its own
//!    properties on the AuthZen [`Subject`] (`mestier.user_id`,
//!    `mestier.member_id`, `mestier.permissions`, `mestier.organization_id`).
//!    Centralizing them here keeps the format consistent and lets the
//!    [`LocalPolicyEngine`](authz::LocalPolicyEngine) read them with
//!    matching constants.
//!
//! 2. **Enrichment & decision helpers** — load the actor's organization
//!    membership + aggregated permission bitfield, then call
//!    [`Authorizer::evaluate`] with a properly shaped request and map
//!    the [`Decision`] to a [`CoreError`].

use authz::{
    AccessEvaluationRequest, Action, Authorizer, Resource, SUBJECT_IAM_ROLES_KEY,
    SUBJECT_PERMISSIONS_KEY, Subject,
};
use common::CoreError;
use serde_json::json;

use crate::{
    UserId,
    domain::{
        member::ports::MemberRepository,
        organization::OrganizationId,
        role::{Permissions, ports::RoleRepository},
    },
};

/// Subject property carrying the actor's Mestier `UserId` (UUID string).
pub const SUBJECT_USER_ID_KEY: &str = "mestier.user_id";

/// Subject property carrying the actor's `MemberId` for the org context.
pub const SUBJECT_MEMBER_ID_KEY: &str = "mestier.member_id";

/// Subject property carrying the `OrganizationId` of the org context.
pub const SUBJECT_ORGANIZATION_ID_KEY: &str = "mestier.organization_id";

/// Re-exported for callers that build subjects without going through
/// [`enrich_for_organization`] (e.g. unit tests).
pub use authz::SUBJECT_PERMISSIONS_KEY as SUBJECT_PERMISSIONS_KEY_REEXPORT;

/// Read the Mestier `UserId` carried by a [`Subject`]. Returns
/// [`CoreError::Internal`] when missing or malformed — handlers must
/// always set this property before calling into a service.
pub fn subject_user_id(subject: &Subject) -> Result<UserId, CoreError> {
    use std::str::FromStr;

    let raw = subject
        .properties
        .get(SUBJECT_USER_ID_KEY)
        .and_then(|v| v.as_str())
        .ok_or_else(|| CoreError::Internal(format!("subject missing `{SUBJECT_USER_ID_KEY}`")))?;
    UserId::from_str(raw).map_err(|e| {
        CoreError::Internal(format!(
            "subject `{SUBJECT_USER_ID_KEY}` is not a UUID: {e}"
        ))
    })
}

/// Loads `(member, aggregated permissions)` for the user behind the
/// subject in the given organization, and writes the AuthZen-shaped
/// properties (`mestier.member_id`, `mestier.organization_id`,
/// `mestier.permissions`) onto a clone of the subject.
///
/// Returns [`CoreError::Forbidden`] when the user is not a member of the
/// organization — the actor has authenticated but has no standing in
/// this org, which is an authorization failure (not a 404 — that would
/// leak existence). System subjects are short-circuited (no DB load).
pub async fn enrich_for_organization<M, R>(
    subject: Subject,
    organization_id: OrganizationId,
    members: &mut M,
    roles: &mut R,
) -> Result<Subject, CoreError>
where
    M: MemberRepository,
    R: RoleRepository,
{
    if subject.is_system() {
        return Ok(subject);
    }

    let user_id = subject_user_id(&subject)?;

    let member = members
        .find_by_org_and_user(organization_id, user_id)
        .await?
        .ok_or(CoreError::Forbidden {
            reason: Some("not a member of this organization".to_owned()),
        })?;

    let role_ids = members.list_role_ids(member.id).await?;
    let org_roles = roles.list_by_organization(organization_id).await?;

    let aggregated = org_roles
        .iter()
        .filter(|r| role_ids.contains(&r.id))
        .map(|r| r.permissions)
        .fold(Permissions::NONE, |acc, p| acc | p);

    let mut enriched = subject;
    enriched.properties.insert(
        SUBJECT_MEMBER_ID_KEY.to_owned(),
        json!(member.id.0.to_string()),
    );
    enriched.properties.insert(
        SUBJECT_ORGANIZATION_ID_KEY.to_owned(),
        json!(organization_id.0.to_string()),
    );
    enriched
        .properties
        .insert(SUBJECT_PERMISSIONS_KEY.to_owned(), json!(aggregated.bits()));

    Ok(enriched)
}

/// Computes the aggregated [`Permissions`] for a member from their org-role set.
///
/// Mirrors the OR-fold inside [`enrich_for_organization`] but as a pure, sync
/// helper so callers that already hold the member + role data (e.g. the channel
/// permission resolver) can reuse it without a redundant DB round-trip.
///
/// `member_role_ids` is the set of role IDs held by the member (from
/// [`MemberRepository::list_role_ids`]).  `org_roles` is the full set of
/// [`Role`]s for the organization (from [`RoleRepository::list_by_organization`]).
pub fn resolve_org_permissions(
    _member: &crate::domain::member::Member,
    member_role_ids: &[crate::domain::role::RoleId],
    org_roles: &[crate::domain::role::Role],
) -> Permissions {
    org_roles
        .iter()
        .filter(|r| member_role_ids.contains(&r.id))
        .map(|r| r.permissions)
        .fold(Permissions::NONE, |acc, p| acc | p)
}

/// Calls the authorizer with `(subject, action, resource)` and converts
/// a deny decision into [`CoreError::Forbidden`]. PDP-side failures
/// surface as [`CoreError::Internal`] (the PDP itself is misbehaving;
/// that should not look like a denial to the caller).
pub async fn require<A: Authorizer>(
    authorizer: &A,
    subject: &Subject,
    action: &str,
    resource: Resource,
) -> Result<(), CoreError> {
    let request = AccessEvaluationRequest::new(subject.clone(), Action::new(action), resource);

    let decision = authorizer
        .evaluate(&request)
        .await
        .map_err(|e| CoreError::Internal(format!("authorization engine error: {e}")))?;

    if decision.is_allowed() {
        return Ok(());
    }

    let reason = decision
        .context
        .as_ref()
        .and_then(|c| c.0.get("reason"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_owned());

    Err(CoreError::Forbidden { reason })
}

/// Builds a [`Subject`] for a regular Mestier user. `iam_roles` is the list
/// of realm/IAM roles carried by the JWT, surfaced under `iam.roles`
/// where the [`LocalPolicyEngine`](authz::LocalPolicyEngine) reads them
/// for the super-admin bypass.
pub fn user_subject(user_id: UserId, iam_roles: Vec<String>) -> Subject {
    use authz::SubjectKind;

    Subject::new(SubjectKind::User, user_id.to_string())
        .with_property(SUBJECT_USER_ID_KEY, user_id.to_string())
        .with_property(SUBJECT_IAM_ROLES_KEY, json!(iam_roles))
}

impl crate::application::MestierUseCase {
    /// The caller's aggregated [`Permissions`] in one organization — a
    /// read, never a refusal. Reused wherever a response has to *change
    /// shape* by capability rather than be refused outright: #306's
    /// profitability redaction is the first caller, deciding per request
    /// whether to null the money fields, not whether to serve the report
    /// at all.
    ///
    /// Same three-step load [`resolve_channel_permissions`] and
    /// [`list_visible_channels`] already do for chat: member, their role
    /// ids, the org's roles, OR-folded. `CoreError::Forbidden` when the
    /// caller has no membership at all — callers that already checked
    /// membership can still call this safely, it just repeats a cheap read.
    #[mestier_macros::transactional(member, role)]
    pub async fn member_permissions(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
    ) -> Result<Permissions, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let member = member_repository
            .find_by_org_and_user(organization_id, user_id)
            .await?
            .ok_or(CoreError::Forbidden {
                reason: Some("not a member of this organization".to_owned()),
            })?;

        let member_role_ids = member_repository.list_role_ids(member.id).await?;
        let org_roles = role_repository
            .list_by_organization(organization_id)
            .await?;

        Ok(resolve_org_permissions(
            &member,
            &member_role_ids,
            &org_roles,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::member::{Member, MemberId};
    use crate::domain::role::Role;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_role(org_id: OrganizationId, perms: Permissions) -> Role {
        use crate::domain::role::RoleId;
        Role {
            id: RoleId(Uuid::new_v4()),
            organization_id: org_id,
            name: "test".to_string(),
            permissions: perms,
            is_seeded: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_member(org_id: OrganizationId, user_id: UserId) -> Member {
        Member {
            id: MemberId(Uuid::new_v4()),
            organization_id: org_id,
            user_id: Some(user_id),
            last_name: "Member".to_owned(),
            first_name: None,
            joined_at: Some(Utc::now()),
            created_at: Utc::now(),
            deleted_at: None,
        }
    }

    #[test]
    fn resolve_org_permissions_ors_role_bits() {
        use crate::domain::role::RoleId;
        let org = OrganizationId(Uuid::from_u128(1));
        let user = UserId(Uuid::from_u128(2));
        let member = make_member(org, user);
        let roles = vec![
            make_role(org, Permissions::MANAGE_ORG),
            make_role(org, Permissions::MANAGE_MEMBERS),
        ];
        let role_ids: Vec<RoleId> = roles.iter().map(|r| r.id).collect();

        let result = resolve_org_permissions(&member, &role_ids, &roles);

        assert!(result.contains(Permissions::MANAGE_ORG));
        assert!(result.contains(Permissions::MANAGE_MEMBERS));
        assert!(!result.contains(Permissions::MANAGE_ROLES));
    }

    #[test]
    fn resolve_org_permissions_ignores_roles_not_held() {
        use crate::domain::role::RoleId;
        let org = OrganizationId(Uuid::from_u128(1));
        let user = UserId(Uuid::from_u128(2));
        let member = make_member(org, user);
        let other_role_id = RoleId(Uuid::from_u128(99));
        let roles = vec![make_role(org, Permissions::MANAGE_ORG)];
        // member holds other_role_id, which is NOT in `roles` list — yields NONE
        let result = resolve_org_permissions(&member, &[other_role_id], &roles);
        assert!(result.is_empty());
    }

    #[test]
    fn resolve_org_permissions_no_roles_yields_none() {
        let org = OrganizationId(Uuid::from_u128(1));
        let user = UserId(Uuid::from_u128(2));
        let member = make_member(org, user);
        let result = resolve_org_permissions(&member, &[], &[]);
        assert_eq!(result, Permissions::NONE);
    }
}

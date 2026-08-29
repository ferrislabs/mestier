use chrono::{DateTime, Utc};
use mestier_core::{Invitation, InvitationId, Member, MemberId, MemberWithAccount, OrganizationId};
use serde::Serialize;
use utoipa::ToSchema;

/// Who occupies a seat, projected for HTTP. Deliberately narrow: this is the
/// one place `users.id`/`users.sub` could leak into a member response, and
/// the chantier this API belongs to exists specifically to stop that leak —
/// see `MemberResponse::account`.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct MemberAccountResponse {
    pub email: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct MemberResponse {
    pub id: MemberId,
    pub organization_id: OrganizationId,
    pub last_name: String,
    /// `null` means "not provided" — see `Member::first_name`.
    pub first_name: Option<String>,
    /// `"{last_name} {first_name}"`, from `Member::display_name`.
    pub display_name: String,
    /// `None` when the seat is vacant: nobody holds it yet. Never carries
    /// `users.id`/`users.sub` — see [`MemberAccountResponse`].
    pub account: Option<MemberAccountResponse>,
    pub joined_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<Member> for MemberResponse {
    /// Maps the seat's own fields only — `account` starts `None` even for
    /// an occupied seat. Callers resolve the occupant (`find_user_by_id`
    /// for an item route, `list_users_by_ids` for a page) and set `account`
    /// afterward; this conversion never touches `UserRepository` itself, so
    /// it stays a plain, synchronous, unit-testable mapping.
    fn from(value: Member) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            display_name: value.display_name(),
            last_name: value.last_name,
            first_name: value.first_name,
            account: None,
            joined_at: value.joined_at,
            created_at: value.created_at,
        }
    }
}

impl From<MemberWithAccount> for MemberResponse {
    /// The list/get-one counterpart of `From<Member>`: the occupant is
    /// already hydrated by `MemberService`, so this conversion is still a
    /// plain, synchronous mapping — it just has one more field to fill in.
    fn from(value: MemberWithAccount) -> Self {
        let account = value.account.map(|user| MemberAccountResponse {
            email: user.email,
            name: user.name,
        });
        let mut response = MemberResponse::from(value.member);
        response.account = account;
        response
    }
}

/// A pending invitation, projected for HTTP. Never carries the token or its
/// hash — see `CreatedInvitationResponse` for the one response that does
/// carry the clear value, and only once.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct InvitationResponse {
    pub id: InvitationId,
    pub organization_id: OrganizationId,
    /// `None` when acceptance will create the seat itself — see
    /// `Invitation::member_id`.
    pub member_id: Option<MemberId>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl From<Invitation> for InvitationResponse {
    fn from(value: Invitation) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            member_id: value.member_id,
            expires_at: value.expires_at,
            created_at: value.created_at,
        }
    }
}

/// The one response that ever carries the clear token — mirrors
/// `handlers_automation::response::CredentialWithSecretResponse`, the same
/// shape for the same reason: a secret shown once, flattened alongside the
/// resource it belongs to rather than nested under it.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct CreatedInvitationResponse {
    #[serde(flatten)]
    pub invitation: InvitationResponse,
    pub token: String,
}

/// The caller's own granted permission bits, by name (#307) — a plain list
/// rather than one boolean field per bit, so a bit added later shows up
/// here without a schema change on this response. See
/// `mestier_core::domain::role::Permissions::NAMED` for where the set of
/// possible names comes from.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct MyPermissionsResponse {
    pub permissions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    // `uuid` is not a direct dependency of `handlers-member` (a `Cargo.toml`
    // this workstream mirrors from `handlers-reference` without adding to),
    // so fixture ids are parsed from literal strings via `FromStr` rather
    // than generated — see `handlers-reference/src/absence/mod.rs` tests.
    fn member() -> Member {
        let now = Utc::now();
        let id: MemberId = "11111111-1111-1111-1111-111111111111".parse().unwrap();
        let organization_id: OrganizationId =
            "22222222-2222-2222-2222-222222222222".parse().unwrap();
        Member {
            id,
            organization_id,
            user_id: None,
            last_name: "Parmantier".to_owned(),
            first_name: Some("Baptiste".to_owned()),
            joined_at: Some(now),
            created_at: now,
            deleted_at: None,
        }
    }

    #[test]
    fn member_response_maps_core_fields_and_computes_display_name() {
        let source = member();
        let expected_id = source.id;
        let expected_org = source.organization_id;
        let expected_joined_at = source.joined_at;
        let expected_created_at = source.created_at;

        let response: MemberResponse = source.into();

        assert_eq!(response.id, expected_id);
        assert_eq!(response.organization_id, expected_org);
        assert_eq!(response.last_name, "Parmantier");
        assert_eq!(response.first_name.as_deref(), Some("Baptiste"));
        assert_eq!(response.display_name, "Parmantier Baptiste");
        assert_eq!(response.joined_at, expected_joined_at);
        assert_eq!(response.created_at, expected_created_at);
    }

    #[test]
    fn member_response_account_starts_absent_regardless_of_seat_occupancy() {
        let mut source = member();
        source.user_id = Some("33333333-3333-3333-3333-333333333333".parse().unwrap());

        let response: MemberResponse = source.into();

        assert_eq!(response.account, None);
    }

    /// The invariant this chantier exists to enforce: a member response's
    /// account projection never carries `users.id` or `users.sub`. Locking
    /// the serialized shape to exactly `{email, name}` means adding either
    /// field back would fail this test rather than silently leak through.
    #[test]
    fn member_account_response_serializes_with_only_email_and_name() {
        let account = MemberAccountResponse {
            email: "baptiste@example.com".to_owned(),
            name: "Baptiste Parmantier".to_owned(),
        };

        let json = serde_json::to_value(&account).unwrap();
        let mut keys: Vec<&str> = json
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();

        assert_eq!(keys, vec!["email", "name"]);
    }
}

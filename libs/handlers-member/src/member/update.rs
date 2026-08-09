use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::UpdateMemberCommand;
use serde::{Deserialize, Deserializer};
use utoipa::ToSchema;

use crate::{paths::MemberPath, require_permission, resolve_account, response::MemberResponse};

/// Distinguishes "the key is absent" (leave `first_name` unchanged) from
/// "the key is present" (apply it, `null` included) — plain
/// `Option<Option<T>>` cannot make that distinction on its own because serde
/// treats a missing key and an explicit `null` the same way by default.
/// Copied from `libs/handlers-reference/src/absence/update.rs`.
fn deserialize_present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}

/// A real PATCH: a field left out of the body leaves the corresponding
/// column untouched. `last_name` uses a plain `Option<String>` (there is no
/// "clear the last name" — a seat always has one). `first_name` needs the
/// three-way distinction between "not provided", "clear it" (`null`) and
/// "set it" — see [`deserialize_present`]. There is deliberately no
/// `user_id` field: occupying or vacating a seat is an invitation concern
/// (#184), not a rename.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateMemberRequest {
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_present")]
    #[schema(value_type = Option<String>, nullable)]
    pub first_name: Option<Option<String>>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/members/{member_id}",
    operation_id = "updateMember",
    tag = super::super::TAG,
    params(
        ("member_id" = mestier_core::MemberId, Path, description = "Member identifier"),
    ),
    request_body = UpdateMemberRequest,
    responses(
        (status = 200, description = "Member updated", body = inline(DataEnvelope<MemberResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Member not found"),
        (status = 409, description = "Member conflict"),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, err)]
pub async fn handler(
    MemberPath { member_id }: MemberPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateMemberRequest>,
) -> Result<Response<MemberResponse>, ApiError> {
    // Load first, then check the organization from the loaded resource —
    // never from the path — so a member from another organization is a 403
    // rather than an IDOR that leaks its data.
    let current = state.usecase.get_member(member_id).await?;
    require_permission(&state, &identity, current.organization_id, "member.manage").await?;
    let actor = resolve_user_id(&state, &identity).await?;

    // `UpdateMemberCommand` takes final values, not a patch — the read side
    // of the PATCH semantics lives here: an absent field falls back to the
    // value already on the loaded member instead of being reset.
    let last_name = payload
        .last_name
        .unwrap_or_else(|| current.last_name.clone());
    let first_name = match payload.first_name {
        Some(value) => value,
        None => current.first_name.clone(),
    };

    let member = state
        .usecase
        .acting_as(actor)
        .update_member(UpdateMemberCommand {
            member_id,
            last_name,
            first_name,
        })
        .await?;

    let account = resolve_account(&state, member.user_id).await?;
    let mut response = MemberResponse::from(member);
    response.account = account;

    Ok(Response::OK(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse(value: serde_json::Value) -> UpdateMemberRequest {
        serde_json::from_value(value).expect("payload must deserialize")
    }

    #[test]
    fn absent_last_name_leaves_it_unset() {
        let request = parse(json!({}));

        assert_eq!(request.last_name, None);
    }

    #[test]
    fn present_last_name_sets_the_new_value() {
        let request = parse(json!({ "last_name": "Bonnal" }));

        assert_eq!(request.last_name, Some("Bonnal".to_owned()));
    }

    #[test]
    fn absent_first_name_leaves_it_unset() {
        let request = parse(json!({}));

        assert_eq!(
            request.first_name, None,
            "an absent `first_name` key must not be mistaken for a clearing `null`"
        );
    }

    #[test]
    fn null_first_name_clears_it() {
        let request = parse(json!({ "first_name": null }));

        assert_eq!(
            request.first_name,
            Some(None),
            "an explicit `null` must be distinguishable from an absent key: it clears the first name"
        );
    }

    #[test]
    fn present_first_name_sets_the_new_value() {
        let request = parse(json!({ "first_name": "Baptiste" }));

        assert_eq!(request.first_name, Some(Some("Baptiste".to_owned())));
    }

    #[test]
    fn empty_body_deserializes_with_every_field_absent() {
        let request = parse(json!({}));

        assert_eq!(request.last_name, None);
        assert_eq!(request.first_name, None);
    }
}

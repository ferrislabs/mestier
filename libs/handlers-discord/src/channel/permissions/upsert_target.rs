use auth::Identity;
use axum::{Extension, Json, extract::State};
use common::{RoleId, UserId};
use discord::{OverwriteTarget, UpsertChannelOverwrite};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{
    paths::ChannelOverwriteTargetPath,
    require_permission,
    response::{OverwriteResponse, UpsertOverwriteRequest},
};

fn parse_target(target_type: &str, target_id: uuid::Uuid) -> Result<OverwriteTarget, ApiError> {
    match target_type {
        "role" => Ok(OverwriteTarget::Role(RoleId(target_id))),
        "member" => Ok(OverwriteTarget::Member(UserId(target_id))),
        other => Err(ApiError::Validation(format!(
            "invalid target_type '{}': must be 'role' or 'member'",
            other
        ))),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/chat/channels/{channel_id}/permissions/{target_type}/{target_id}",
    operation_id = "upsertTargetOverwrite",
    tag = super::super::super::TAG,
    params(
        ("channel_id"  = discord::ChannelId, Path, description = "Channel identifier"),
        ("target_type" = String, Path, description = "Overwrite target: 'role' or 'member'"),
        ("target_id"   = uuid::Uuid, Path, description = "Role or member UUID"),
    ),
    request_body = UpsertOverwriteRequest,
    responses(
        (status = 200, description = "Role or member overwrite upserted", body = inline(DataEnvelope<OverwriteResponse>)),
        (status = 400, description = "Invalid target_type"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden — requires org-level MANAGE_CHANNELS"),
        (status = 404, description = "Channel not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ChannelOverwriteTargetPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpsertOverwriteRequest>,
) -> Result<Response<OverwriteResponse>, ApiError> {
    let channel = state.usecase.get_channel(path.channel_id).await?;
    require_permission(&state, &identity, channel.organization_id, "channel.manage").await?;

    let target = parse_target(&path.target_type, path.target_id)?;

    let overwrite = state
        .usecase
        .upsert_channel_overwrite(UpsertChannelOverwrite {
            channel_id: path.channel_id,
            organization_id: channel.organization_id,
            target,
            allow: payload.allow,
            deny: payload.deny,
        })
        .await?;

    Ok(Response::OK(OverwriteResponse::from(overwrite)))
}

#[cfg(test)]
mod tests {
    use super::parse_target;
    use discord::OverwriteTarget;
    use uuid::Uuid;

    const TARGET_UUID_STR: &str = "550e8400-e29b-41d4-a716-446655440000";

    #[test]
    fn parse_target_role_returns_role_variant() {
        let uuid = Uuid::parse_str(TARGET_UUID_STR).unwrap();
        let target = parse_target("role", uuid).expect("role must parse");
        assert!(
            matches!(target, OverwriteTarget::Role(_)),
            "expected Role variant; got {target:?}"
        );
    }

    #[test]
    fn parse_target_member_returns_member_variant() {
        let uuid = Uuid::parse_str(TARGET_UUID_STR).unwrap();
        let target = parse_target("member", uuid).expect("member must parse");
        assert!(
            matches!(target, OverwriteTarget::Member(_)),
            "expected Member variant; got {target:?}"
        );
    }

    #[test]
    fn parse_target_everyone_string_returns_400() {
        // "everyone" is not valid for the typed-target path — it has its own path segment.
        let uuid = Uuid::parse_str(TARGET_UUID_STR).unwrap();
        let err = parse_target("everyone", uuid)
            .expect_err("'everyone' on typed-target path must return Err");
        // ApiError::Validation wraps the message
        let msg = format!("{err:?}");
        assert!(
            msg.contains("invalid target_type"),
            "error must describe invalid target_type; got: {msg}"
        );
    }

    #[test]
    fn parse_target_garbage_string_returns_400() {
        let uuid = Uuid::parse_str(TARGET_UUID_STR).unwrap();
        let err = parse_target("channel", uuid).expect_err("garbage type must return Err");
        let msg = format!("{err:?}");
        assert!(msg.contains("invalid target_type"), "got: {msg}");
    }
}

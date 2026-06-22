use auth::Identity;
use axum::{Extension, extract::State};
use common::{RoleId, UserId};
use discord::{DeleteChannelOverwrite, OverwriteTarget};
use handlers::{ApiError, AppState, Response};

use crate::{EmptyResponse, paths::ChannelOverwriteTargetPath, require_permission};

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
    delete,
    path = "/api/v1/chat/channels/{channel_id}/permissions/{target_type}/{target_id}",
    operation_id = "deleteTargetOverwrite",
    tag = super::super::super::TAG,
    params(
        ("channel_id"  = discord::ChannelId, Path, description = "Channel identifier"),
        ("target_type" = String, Path, description = "Overwrite target: 'role' or 'member'"),
        ("target_id"   = uuid::Uuid, Path, description = "Role or member UUID"),
    ),
    responses(
        (status = 204, description = "Role or member overwrite deleted"),
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
) -> Result<Response<EmptyResponse>, ApiError> {
    let channel = state.usecase.get_channel(path.channel_id).await?;
    require_permission(&state, &identity, channel.organization_id, "channel.manage").await?;

    let target = parse_target(&path.target_type, path.target_id)?;

    state
        .usecase
        .delete_channel_overwrite(DeleteChannelOverwrite {
            channel_id: path.channel_id,
            target,
        })
        .await?;

    Ok(Response::NoContent)
}

#[cfg(test)]
mod tests {
    use super::parse_target;
    use discord::OverwriteTarget;
    use uuid::Uuid;

    const TARGET_UUID_STR: &str = "550e8400-e29b-41d4-a716-446655440001";

    #[test]
    fn delete_parse_target_role_returns_role_variant() {
        let uuid = Uuid::parse_str(TARGET_UUID_STR).unwrap();
        let target = parse_target("role", uuid).expect("role must parse");
        assert!(matches!(target, OverwriteTarget::Role(_)));
    }

    #[test]
    fn delete_parse_target_member_returns_member_variant() {
        let uuid = Uuid::parse_str(TARGET_UUID_STR).unwrap();
        let target = parse_target("member", uuid).expect("member must parse");
        assert!(matches!(target, OverwriteTarget::Member(_)));
    }

    #[test]
    fn delete_parse_target_invalid_string_returns_400() {
        let uuid = Uuid::parse_str(TARGET_UUID_STR).unwrap();
        let err = parse_target("admin", uuid).expect_err("invalid type must fail");
        let msg = format!("{err:?}");
        assert!(msg.contains("invalid target_type"), "got: {msg}");
    }
}

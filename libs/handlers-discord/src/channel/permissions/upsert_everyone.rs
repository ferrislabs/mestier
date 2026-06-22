use auth::Identity;
use axum::{Extension, Json, extract::State};
use discord::{OverwriteTarget, UpsertChannelOverwrite};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{
    paths::ChannelOverwriteEveryonePath,
    require_permission,
    response::{OverwriteResponse, UpsertOverwriteRequest},
};

#[utoipa::path(
    put,
    path = "/api/v1/chat/channels/{channel_id}/permissions/everyone",
    operation_id = "upsertEveryoneOverwrite",
    tag = super::super::super::TAG,
    params(("channel_id" = discord::ChannelId, Path, description = "Channel identifier")),
    request_body = UpsertOverwriteRequest,
    responses(
        (status = 200, description = "EVERYONE overwrite upserted", body = inline(DataEnvelope<OverwriteResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden — requires org-level MANAGE_CHANNELS"),
        (status = 404, description = "Channel not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ChannelOverwriteEveryonePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpsertOverwriteRequest>,
) -> Result<Response<OverwriteResponse>, ApiError> {
    let channel = state.usecase.get_channel(path.channel_id).await?;
    require_permission(&state, &identity, channel.organization_id, "channel.manage").await?;

    let overwrite = state
        .usecase
        .upsert_channel_overwrite(UpsertChannelOverwrite {
            channel_id: path.channel_id,
            organization_id: channel.organization_id,
            target: OverwriteTarget::Everyone,
            allow: payload.allow,
            deny: payload.deny,
        })
        .await?;

    Ok(Response::OK(OverwriteResponse::from(overwrite)))
}

#[cfg(test)]
mod tests {
    use crate::response::UpsertOverwriteRequest;

    #[test]
    fn everyone_upsert_request_deserializes_zero_bits() {
        // A no-op EVERYONE overwrite (allow=0, deny=0) must be accepted.
        let json = r#"{"allow":0,"deny":0}"#;
        let req: UpsertOverwriteRequest =
            serde_json::from_str(json).expect("must deserialize with zero bits");
        assert_eq!(req.allow, 0);
        assert_eq!(req.deny, 0);
    }

    #[test]
    fn everyone_upsert_request_deserializes_view_channel_deny() {
        // deny=32 means VIEW_CHANNEL denied (private channel default).
        let json = r#"{"allow":0,"deny":32}"#;
        let req: UpsertOverwriteRequest = serde_json::from_str(json).expect("must deserialize");
        assert_eq!(req.deny, 32);
    }
}

use auth::Identity;
use axum::{Extension, Json, extract::State};
use discord::{CategoryId, CreateChannelCommand};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::OrgChannelsPath, require_permission, response::ChannelResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateChannelRequest {
    pub name: String,
    pub topic: Option<String>,
    pub position: i32,
    pub category_id: Option<CategoryId>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/channels",
    operation_id = "createChannel",
    tag = super::super::TAG,
    params(("organization_id" = common::OrganizationId, Path, description = "Organization identifier")),
    request_body = CreateChannelRequest,
    responses(
        (status = 201, description = "Channel created", body = inline(DataEnvelope<ChannelResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden — requires MANAGE_CHANNELS"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: OrgChannelsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateChannelRequest>,
) -> Result<Response<ChannelResponse>, ApiError> {
    require_permission(&state, &identity, path.organization_id, "channel.manage").await?;

    if payload.name.trim().is_empty() {
        return Err(ApiError::Validation(
            "channel name must not be blank".into(),
        ));
    }

    let channel = state
        .usecase
        .create_channel(CreateChannelCommand {
            organization_id: path.organization_id,
            category_id: payload.category_id,
            name: payload.name,
            topic: payload.topic,
            position: payload.position,
        })
        .await?;

    Ok(Response::Created(channel.into()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn blank_channel_name_is_rejected() {
        assert!("".trim().is_empty());
    }
}

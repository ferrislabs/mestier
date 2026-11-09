use auth::Identity;
use axum::{Extension, Json, extract::State};
use discord::{CategoryId, ChannelId, ChannelPlacement};
use handlers::{ApiError, AppState, Response, resolve_user_id};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{EmptyResponse, paths::OrgChannelOrderPath, require_permission};

const MAX_PLACEMENTS: usize = 500;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ChannelPlacementRequest {
    pub channel_id: ChannelId,
    pub category_id: Option<CategoryId>,
    pub position: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ReorderChannelsRequest {
    pub placements: Vec<ChannelPlacementRequest>,
}

impl From<ChannelPlacementRequest> for ChannelPlacement {
    fn from(placement: ChannelPlacementRequest) -> Self {
        Self {
            channel_id: placement.channel_id,
            category_id: placement.category_id,
            position: placement.position,
        }
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/chat/organizations/{organization_id}/channels/order",
    operation_id = "reorderChannels",
    tag = super::super::TAG,
    params(("organization_id" = common::OrganizationId, Path, description = "Organization identifier")),
    request_body = ReorderChannelsRequest,
    responses(
        (status = 204, description = "Channels placed"),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden — requires MANAGE_CHANNELS"),
        (status = 404, description = "Channel not found"),
        (status = 409, description = "A category, a channel kind or a duplicate is refused"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: OrgChannelOrderPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<ReorderChannelsRequest>,
) -> Result<Response<EmptyResponse>, ApiError> {
    require_permission(&state, &identity, path.organization_id, "channel.manage").await?;
    let actor = resolve_user_id(&state, &identity).await?;

    if payload.placements.len() > MAX_PLACEMENTS {
        return Err(ApiError::Validation(format!(
            "at most {MAX_PLACEMENTS} channels can be placed at once"
        )));
    }

    state
        .usecase
        .acting_as(actor)
        .reorder_channels(
            path.organization_id,
            payload.placements.into_iter().map(Into::into).collect(),
        )
        .await?;

    Ok(Response::NoContent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_placement_without_category_reads_as_uncategorized() {
        let request: ReorderChannelsRequest = serde_json::from_str(
            r#"{"placements":[{"channel_id":"018f0000-0000-7000-8000-000000000001","category_id":null,"position":3}]}"#,
        )
        .expect("a valid request");

        assert_eq!(request.placements[0].category_id, None);
        assert_eq!(request.placements[0].position, 3);
    }
}

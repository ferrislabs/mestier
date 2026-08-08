use auth::Identity;
use axum::{Extension, Router, extract::State, middleware::from_fn_with_state};
use axum_extra::routing::RouterExt;
use handlers::{
    ApiError, AppState, DataEnvelope, Response, auth::auth_middleware,
    rate_limit::rate_limit_middleware,
};
use mestier_core::OrganizationId;
use uuid::Uuid;

pub mod delivery;
pub mod endpoint;
pub mod paths;
pub mod response;
pub mod settings;

pub const TAG: &str = "automation";

#[derive(Debug, serde::Serialize, PartialEq)]
pub struct EmptyResponse;

async fn require_org_membership(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
) -> Result<(), ApiError> {
    let user = state
        .usecase
        .find_user_by_sub(identity.id())
        .await?
        .ok_or(ApiError::Forbidden)?;

    if state
        .usecase
        .find_membership(organization_id, user.id)
        .await?
        .is_none()
    {
        return Err(ApiError::Forbidden);
    }

    Ok(())
}

/// Membership **and** ownership: an endpoint id is guessable, so belonging to
/// the organization in the path is not enough — the endpoint has to belong to
/// it too, or one organization edits another's webhook.
async fn require_endpoint_membership(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
    endpoint_id: Uuid,
) -> Result<(), ApiError> {
    require_org_membership(state, identity, organization_id).await?;

    let (endpoint, _) = state.usecase.get_webhook_endpoint(endpoint_id).await?;
    if endpoint.org_id != organization_id {
        // Not `Forbidden`: telling a caller an id exists elsewhere is itself
        // a leak.
        return Err(ApiError::NotFound);
    }

    Ok(())
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/automation/events",
    operation_id = "listAutomationEvents",
    tag = TAG,
    params(("organization_id" = OrganizationId, Path, description = "Organization identifier")),
    responses(
        (status = 200, description = "Every event an endpoint may subscribe to", body = inline(DataEnvelope<Vec<response::EventDescriptorResponse>>)),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_events(
    path: paths::CataloguePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<response::EventDescriptorResponse>>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    // Served from the catalogue rather than a hand-kept list, so the picker
    // cannot offer an event the server does not know.
    let mut events: Vec<response::EventDescriptorResponse> = mestier_core::event_catalogue()
        .descriptors()
        .map(|descriptor| response::EventDescriptorResponse {
            name: descriptor.name.to_owned(),
            version: descriptor.version,
            label: descriptor.label.to_owned(),
            subject_kind: descriptor.subject_kind.to_owned(),
            payload_example: descriptor.payload_example.clone(),
        })
        .collect();
    events.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(Response::OK(events))
}

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list_events)
        .typed_get(endpoint::list::handler)
        .typed_post(endpoint::create::handler)
        .typed_patch(endpoint::update::handler)
        .typed_delete(endpoint::delete::handler)
        .typed_post(endpoint::regenerate_secret::handler)
        .typed_get(settings::get::handler)
        .typed_patch(settings::update::handler)
        .typed_get(delivery::list::handler)
        .typed_post(delivery::replay::handler)
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}

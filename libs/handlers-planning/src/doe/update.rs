use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::NaiveDate;
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{DoeParty, DoeSection, Revision, UpdateDoeCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::doe::{DoePath, require_doe, response::DoeResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateDoeRequest {
    pub title: String,
    #[serde(default)]
    pub place: Option<String>,
    #[serde(default)]
    pub lot: Option<String>,
    pub issued_on: NaiveDate,
    #[serde(default)]
    pub author: Option<String>,
    /// 1 to 8 characters.
    pub revision: String,
    pub parties: Vec<DoeParty>,
    pub sections: Vec<DoeSection>,
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/does/{doe_id}",
    operation_id = "updateDoe",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("doe_id" = mestier_core::DoeId, Path, description = "DOE identifier"),
    ),
    request_body = UpdateDoeRequest,
    responses(
        (status = 200, description = "The DOE, replaced", body = inline(DataEnvelope<DoeResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "DOE not found"),
        (status = 409, description = "A blank title, section title, designation or party name, an invalid revision, or more than 30 sections"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    DoePath {
        organization_id,
        doe_id,
    }: DoePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateDoeRequest>,
) -> Result<Response<DoeResponse>, ApiError> {
    require_doe(&state, &identity, organization_id, doe_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;

    let doe = state
        .usecase
        .update_doe(UpdateDoeCommand {
            actor,
            id: doe_id,
            title: payload.title,
            place: payload.place,
            lot: payload.lot,
            issued_on: payload.issued_on,
            author: payload.author,
            revision: Revision::try_from(payload.revision)?,
            parties: payload.parties,
            sections: payload.sections,
        })
        .await?;

    Ok(Response::OK(doe.into()))
}

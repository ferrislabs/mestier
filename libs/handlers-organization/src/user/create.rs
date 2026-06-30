use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use iam::{IamCreateUser, IamError, IamProvider};
use mestier_core::{AddMemberCommand, CreateUserCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::OrganizationUsersPath,
    require_org_membership,
    response::UserResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateOrgUserRequest {
    pub email: String,
    pub username: String,
    pub name: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/users",
    operation_id = "createOrganizationUser",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateOrgUserRequest,
    responses(
        (status = 201, description = "User created and added to organization", body = inline(DataEnvelope<UserResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden — not a member of this organization"),
        (status = 409, description = "Email already registered in FerrisKey"),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, fields(organization_id = %path.organization_id.0, email = %payload.email), err)]
pub async fn handler(
    path: OrganizationUsersPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateOrgUserRequest>,
) -> Result<Response<UserResponse>, ApiError> {
    // Verify caller is a member of the org (admin gate).
    require_org_membership(&state, &identity, path.organization_id).await?;

    // 1. Create the user in FerrisKey.
    let iam_user = state
        .iam
        .create_user(IamCreateUser {
            email: payload.email.clone(),
            username: payload.username.clone(),
            name: Some(payload.name.clone()),
            send_invite_email: false,
        })
        .await
        .map_err(|e| match e {
            IamError::Conflict(_) => ApiError::Conflict(format!(
                "email '{}' is already registered in FerrisKey",
                payload.email
            )),
            IamError::InvalidInput(msg) => ApiError::BadRequest(msg),
            IamError::Unauthorized | IamError::Forbidden => {
                tracing::error!("IAM credentials rejected while creating user");
                ApiError::Internal
            }
            IamError::Unavailable(msg) => {
                tracing::error!(error = %msg, "IAM unavailable");
                ApiError::ExternalService("identity provider unavailable".into())
            }
            other => {
                tracing::error!(error = %other, "unexpected IAM error");
                ApiError::Internal
            }
        })?;

    // 2. Persist the Mestier users row (upsert on email — idempotent).
    let mestier_user = state
        .usecase
        .create_user(CreateUserCommand {
            name: payload.name,
            username: payload.username,
            email: payload.email,
            sub: iam_user.id.0,
        })
        .await?;

    // 3. Add the new user to the organization as a member.
    state
        .usecase
        .add_member(AddMemberCommand {
            organization_id: path.organization_id,
            user_id: mestier_user.id,
        })
        .await?;

    Ok(Response::Created(mestier_user.into()))
}

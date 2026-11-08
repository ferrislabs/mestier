use auth::Identity;
use axum::{
    Extension, Json, Router,
    extract::State,
    response::{IntoResponse, Response as AxumResponse},
};
use axum_extra::routing::{RouterExt, TypedPath};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use http::StatusCode;
use mestier_core::{
    Acquisition, AcquisitionDraft, ArchiveLeaveAccountCommand, CreateLeaveAccountCommand,
    LeaveAccount, LeaveAccountDraft, LeaveAccountError, LeaveAccountId, ListLeaveAccountsCommand,
    OrganizationId, UpdateLeaveAccountCommand,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

use crate::require_org_membership;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list_accounts)
        .typed_post(create_account)
        .typed_put(update_account)
        .typed_post(archive_account)
        .typed_post(restore_account)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/leave-accounts")]
pub struct LeaveAccountsPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/leave-accounts/{account_id}")]
pub struct LeaveAccountPath {
    pub organization_id: OrganizationId,
    pub account_id: LeaveAccountId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/leave-accounts/{account_id}/archive")]
pub struct ArchiveLeaveAccountPath {
    pub organization_id: OrganizationId,
    pub account_id: LeaveAccountId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/leave-accounts/{account_id}/restore")]
pub struct RestoreLeaveAccountPath {
    pub organization_id: OrganizationId,
    pub account_id: LeaveAccountId,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AcquisitionBody {
    Monthly { per_month: f64 },
    Annual { days: f64 },
}

impl From<AcquisitionBody> for AcquisitionDraft {
    fn from(value: AcquisitionBody) -> Self {
        match value {
            AcquisitionBody::Monthly { per_month } => Self::Monthly { per_month },
            AcquisitionBody::Annual { days } => Self::Annual { days },
        }
    }
}

impl From<Acquisition> for AcquisitionBody {
    fn from(value: Acquisition) -> Self {
        let days = value.amount().days().as_decimal();
        match value {
            Acquisition::Monthly(_) => Self::Monthly { per_month: days },
            Acquisition::Annual(_) => Self::Annual { days },
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LeaveAccountRequest {
    pub name: String,
    pub acquisition: AcquisitionBody,
    pub period_start_month: u16,
    pub carry_over_max: f64,
}

impl From<LeaveAccountRequest> for LeaveAccountDraft {
    fn from(value: LeaveAccountRequest) -> Self {
        Self {
            name: value.name,
            acquisition: value.acquisition.into(),
            period_start_month: u8::try_from(value.period_start_month).unwrap_or(0),
            carry_over_max: value.carry_over_max,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct LeaveAccountResponse {
    pub id: LeaveAccountId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub acquisition: AcquisitionBody,
    pub period_start_month: u8,
    pub carry_over_max: f64,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<LeaveAccount> for LeaveAccountResponse {
    fn from(value: LeaveAccount) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            name: value.name.as_str().to_owned(),
            acquisition: value.acquisition.into(),
            period_start_month: value.period_start_month.get(),
            carry_over_max: value.carry_over_max.days().as_decimal(),
            archived_at: value.archived_at,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug)]
pub enum LeaveAccountFailure {
    Api(ApiError),
    Refused(LeaveAccountError),
}

impl From<ApiError> for LeaveAccountFailure {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<LeaveAccountError> for LeaveAccountFailure {
    fn from(error: LeaveAccountError) -> Self {
        Self::Refused(error)
    }
}

impl IntoResponse for LeaveAccountFailure {
    fn into_response(self) -> AxumResponse {
        match self {
            Self::Api(error) => error.into_response(),
            Self::Refused(error) => {
                let status = StatusCode::UNPROCESSABLE_ENTITY;
                (
                    status,
                    Json(json!({
                        "code": "E_UNPROCESSABLE_ENTITY",
                        "message": error.to_string(),
                        "status": status.as_u16(),
                        "details": { "field": error.field() },
                    })),
                )
                    .into_response()
            }
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/leave-accounts",
    operation_id = "listLeaveAccounts",
    tag = super::TAG,
    params(("organization_id" = OrganizationId, Path, description = "Organization identifier")),
    responses(
        (status = 200, description = "Every leave account, archived ones included; the defaults are created on the first read", body = inline(DataEnvelope<Vec<LeaveAccountResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_accounts(
    path: LeaveAccountsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<LeaveAccountResponse>>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let accounts = state
        .usecase
        .acting_as(user_id)
        .list_leave_accounts(ListLeaveAccountsCommand {
            actor,
            organization_id: path.organization_id,
        })
        .await?;

    Ok(Response::OK(accounts.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/leave-accounts",
    operation_id = "createLeaveAccount",
    tag = super::TAG,
    params(("organization_id" = OrganizationId, Path, description = "Organization identifier")),
    request_body = LeaveAccountRequest,
    responses(
        (status = 201, description = "Leave account created", body = inline(DataEnvelope<LeaveAccountResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 422, description = "The account is refused, the field is named in details"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_account(
    path: LeaveAccountsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<LeaveAccountRequest>,
) -> Result<Response<LeaveAccountResponse>, LeaveAccountFailure> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let created = state
        .usecase
        .acting_as(user_id)
        .create_leave_account(CreateLeaveAccountCommand {
            actor,
            organization_id: path.organization_id,
            draft: payload.into(),
        })
        .await
        .map_err(ApiError::from)??;

    Ok(Response::Created(created.into()))
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/leave-accounts/{account_id}",
    operation_id = "updateLeaveAccount",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("account_id" = LeaveAccountId, Path, description = "Leave account identifier"),
    ),
    request_body = LeaveAccountRequest,
    responses(
        (status = 200, description = "Leave account replaced", body = inline(DataEnvelope<LeaveAccountResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Leave account not found in the organization"),
        (status = 422, description = "The account is refused, the field is named in details"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_account(
    path: LeaveAccountPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<LeaveAccountRequest>,
) -> Result<Response<LeaveAccountResponse>, LeaveAccountFailure> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let updated = state
        .usecase
        .acting_as(user_id)
        .update_leave_account(UpdateLeaveAccountCommand {
            actor,
            organization_id: path.organization_id,
            id: path.account_id,
            draft: payload.into(),
        })
        .await
        .map_err(ApiError::from)??;

    Ok(Response::OK(updated.into()))
}

async fn set_archived(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
    id: LeaveAccountId,
    archived: bool,
) -> Result<Response<LeaveAccountResponse>, ApiError> {
    let (user_id, actor) = resolve_actor(state, identity).await?;
    let account = state
        .usecase
        .acting_as(user_id)
        .archive_leave_account(ArchiveLeaveAccountCommand {
            actor,
            organization_id,
            id,
            archived,
        })
        .await?;

    Ok(Response::OK(account.into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/leave-accounts/{account_id}/archive",
    operation_id = "archiveLeaveAccount",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("account_id" = LeaveAccountId, Path, description = "Leave account identifier"),
    ),
    responses(
        (status = 200, description = "Leave account archived, past adjustments and types keep it", body = inline(DataEnvelope<LeaveAccountResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Leave account not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn archive_account(
    path: ArchiveLeaveAccountPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<LeaveAccountResponse>, ApiError> {
    set_archived(
        &state,
        &identity,
        path.organization_id,
        path.account_id,
        true,
    )
    .await
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/leave-accounts/{account_id}/restore",
    operation_id = "restoreLeaveAccount",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("account_id" = LeaveAccountId, Path, description = "Leave account identifier"),
    ),
    responses(
        (status = 200, description = "Leave account restored", body = inline(DataEnvelope<LeaveAccountResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Leave account not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn restore_account(
    path: RestoreLeaveAccountPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<LeaveAccountResponse>, ApiError> {
    set_archived(
        &state,
        &identity,
        path.organization_id,
        path.account_id,
        false,
    )
    .await
}

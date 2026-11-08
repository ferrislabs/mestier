use auth::Identity;
use axum::{
    Extension, Json, Router,
    extract::State,
    response::{IntoResponse, Response as AxumResponse},
};
use axum_extra::routing::{RouterExt, TypedPath};
use chrono::NaiveDate;
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use http::StatusCode;
use mestier_core::{
    ContractType, CreateMemberContractCommand, DeleteMemberContractCommand,
    ListMemberContractsCommand, MemberContract, MemberContractError, MemberContractId,
    MemberContractInput, MemberId, OrganizationId, ScheduleInput, UpdateMemberContractCommand,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

use crate::EmptyResponse;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list_organization_contracts)
        .typed_get(list_member_contracts)
        .typed_post(create_contract)
        .typed_put(update_contract)
        .typed_delete(delete_contract)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/member-contracts")]
pub struct OrganizationContractsPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/members/{member_id}/contracts")]
pub struct MemberContractsPath {
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/members/{member_id}/contracts/{contract_id}")]
pub struct MemberContractPath {
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    pub contract_id: MemberContractId,
}

/// A contract and its schedule. `end_date` null means open-ended. The schedule
/// is a tagged union on `kind`: `weekly` (seven days, Monday to Sunday),
/// `rotation` (anchor date and repeating blocks) or `variable` (one default
/// day). Times are `HH:MM`, dates `YYYY-MM-DD`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PutMemberContractRequest {
    #[serde(rename = "type")]
    pub contract_type: ContractType,
    #[schema(value_type = String, format = Date)]
    pub start_date: NaiveDate,
    #[schema(value_type = Option<String>, format = Date)]
    #[serde(default)]
    pub end_date: Option<NaiveDate>,
    #[schema(value_type = Option<String>, format = Date)]
    #[serde(default)]
    pub trial_end_date: Option<NaiveDate>,
    pub schedule: ScheduleInput,
}

impl From<PutMemberContractRequest> for MemberContractInput {
    fn from(value: PutMemberContractRequest) -> Self {
        Self {
            contract_type: value.contract_type,
            start_date: value.start_date,
            end_date: value.end_date,
            trial_end_date: value.trial_end_date,
            schedule: value.schedule,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct MemberContractResponse {
    pub id: MemberContractId,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    #[serde(rename = "type")]
    pub contract_type: ContractType,
    #[schema(value_type = String, format = Date)]
    pub start_date: NaiveDate,
    #[schema(value_type = Option<String>, format = Date)]
    pub end_date: Option<NaiveDate>,
    #[schema(value_type = Option<String>, format = Date)]
    pub trial_end_date: Option<NaiveDate>,
    pub schedule: ScheduleInput,
}

impl From<MemberContract> for MemberContractResponse {
    fn from(value: MemberContract) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            member_id: value.member_id,
            contract_type: value.contract_type,
            start_date: value.period.start(),
            end_date: value.period.end(),
            trial_end_date: value.period.trial_end(),
            schedule: ScheduleInput::from(&value.schedule),
        }
    }
}

#[derive(Debug)]
pub enum ContractFailure {
    Api(ApiError),
    Refused(MemberContractError),
}

impl From<ApiError> for ContractFailure {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<MemberContractError> for ContractFailure {
    fn from(error: MemberContractError) -> Self {
        Self::Refused(error)
    }
}

impl IntoResponse for ContractFailure {
    fn into_response(self) -> AxumResponse {
        match self {
            Self::Api(error) => error.into_response(),
            Self::Refused(error) => {
                let status = StatusCode::UNPROCESSABLE_ENTITY;
                (
                    status,
                    Json(json!({
                        "code": error.code(),
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
    path = "/api/v1/organizations/{organization_id}/member-contracts",
    operation_id = "listOrganizationMemberContracts",
    tag = super::TAG,
    params(("organization_id" = OrganizationId, Path, description = "Organization identifier")),
    responses(
        (status = 200, description = "Every contract of the organization", body = inline(DataEnvelope<Vec<MemberContractResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_organization_contracts(
    path: OrganizationContractsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<MemberContractResponse>>, ApiError> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let contracts = state
        .usecase
        .acting_as(user_id)
        .list_member_contracts(ListMemberContractsCommand {
            actor,
            organization_id: path.organization_id,
            member_id: None,
        })
        .await?;

    Ok(Response::OK(
        contracts.into_iter().map(Into::into).collect(),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/members/{member_id}/contracts",
    operation_id = "listMemberContracts",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("member_id" = MemberId, Path, description = "Member identifier"),
    ),
    responses(
        (status = 200, description = "The contracts of the member, oldest first", body = inline(DataEnvelope<Vec<MemberContractResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Member not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_member_contracts(
    path: MemberContractsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<MemberContractResponse>>, ApiError> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let contracts = state
        .usecase
        .acting_as(user_id)
        .list_member_contracts(ListMemberContractsCommand {
            actor,
            organization_id: path.organization_id,
            member_id: Some(path.member_id),
        })
        .await?;

    Ok(Response::OK(
        contracts.into_iter().map(Into::into).collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/members/{member_id}/contracts",
    operation_id = "createMemberContract",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("member_id" = MemberId, Path, description = "Member identifier"),
    ),
    request_body = PutMemberContractRequest,
    responses(
        (status = 201, description = "Contract created", body = inline(DataEnvelope<MemberContractResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Member not found in the organization"),
        (status = 409, description = "A concurrent write took the same days"),
        (status = 422, description = "The contract is refused, the field is named in details"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_contract(
    path: MemberContractsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<PutMemberContractRequest>,
) -> Result<Response<MemberContractResponse>, ContractFailure> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let contract = state
        .usecase
        .acting_as(user_id)
        .create_member_contract(CreateMemberContractCommand {
            actor,
            organization_id: path.organization_id,
            member_id: path.member_id,
            input: payload.into(),
        })
        .await
        .map_err(ApiError::from)??;

    Ok(Response::Created(contract.into()))
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/members/{member_id}/contracts/{contract_id}",
    operation_id = "updateMemberContract",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("member_id" = MemberId, Path, description = "Member identifier"),
        ("contract_id" = MemberContractId, Path, description = "Contract identifier"),
    ),
    request_body = PutMemberContractRequest,
    responses(
        (status = 200, description = "Contract replaced", body = inline(DataEnvelope<MemberContractResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Contract not found for this member in the organization"),
        (status = 409, description = "A concurrent write took the same days"),
        (status = 422, description = "The contract is refused, the field is named in details"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_contract(
    path: MemberContractPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<PutMemberContractRequest>,
) -> Result<Response<MemberContractResponse>, ContractFailure> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let contract = state
        .usecase
        .acting_as(user_id)
        .update_member_contract(UpdateMemberContractCommand {
            actor,
            organization_id: path.organization_id,
            member_id: path.member_id,
            contract_id: path.contract_id,
            input: payload.into(),
        })
        .await
        .map_err(ApiError::from)??;

    Ok(Response::OK(contract.into()))
}

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/members/{member_id}/contracts/{contract_id}",
    operation_id = "deleteMemberContract",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("member_id" = MemberId, Path, description = "Member identifier"),
        ("contract_id" = MemberContractId, Path, description = "Contract identifier"),
    ),
    responses(
        (status = 204, description = "Contract deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Contract not found for this member in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_contract(
    path: MemberContractPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    state
        .usecase
        .acting_as(user_id)
        .delete_member_contract(DeleteMemberContractCommand {
            actor,
            organization_id: path.organization_id,
            member_id: path.member_id,
            contract_id: path.contract_id,
        })
        .await?;

    Ok(Response::NoContent)
}

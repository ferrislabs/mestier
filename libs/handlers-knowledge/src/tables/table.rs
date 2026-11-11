use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, require_org_membership, resolve_actor};
use http::StatusCode;
use mestier_core::{
    CreateKnowledgeTableCommand, DeleteKnowledgeTableCommand, GetKnowledgeTableCommand,
    ListKnowledgeTablesCommand, UpdateKnowledgeTableCommand,
};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    deserialize_present,
    tables::{
        TablePath, TablesPath, refused,
        response::{KnowledgeTableDetailResponse, KnowledgeTableResponse},
    },
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateKnowledgeTableRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateKnowledgeTableRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_present")]
    #[schema(value_type = Option<String>, nullable)]
    pub description: Option<Option<String>>,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/knowledge/tables",
    operation_id = "listKnowledgeTables",
    tag = crate::TAG,
    params(("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier")),
    responses(
        (status = 200, description = "The tables of the organization, by name", body = inline(DataEnvelope<Vec<KnowledgeTableResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list(
    TablesPath { organization_id }: TablesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<KnowledgeTableResponse>>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;
    let tables = state
        .usecase
        .list_knowledge_tables(ListKnowledgeTablesCommand {
            actor,
            organization_id,
        })
        .await?;
    Ok(Response::OK(tables.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/knowledge/tables",
    operation_id = "createKnowledgeTable",
    tag = crate::TAG,
    params(("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier")),
    request_body = CreateKnowledgeTableRequest,
    responses(
        (status = 201, description = "The table, with its primary text column and a grid view", body = inline(DataEnvelope<KnowledgeTableDetailResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "A table already has this name, whatever the case"),
        (status = 422, description = "Empty or too long name or description, or 200 tables already"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create(
    TablesPath { organization_id }: TablesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateKnowledgeTableRequest>,
) -> Result<Response<KnowledgeTableDetailResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let schema = state
        .usecase
        .acting_as(user_id)
        .create_knowledge_table(CreateKnowledgeTableCommand {
            actor,
            organization_id,
            user_id: Some(user_id),
            name: payload.name,
            description: payload.description,
        })
        .await?
        .map_err(refused)?;
    Ok(Response::Created(schema.into()))
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}",
    operation_id = "getKnowledgeTable",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("table_id" = mestier_core::KnowledgeTableId, Path, description = "Table identifier"),
    ),
    responses(
        (status = 200, description = "The table with its columns and views, in order", body = inline(DataEnvelope<KnowledgeTableDetailResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Table not found, or held by another organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get(
    TablePath {
        organization_id,
        table_id,
    }: TablePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<KnowledgeTableDetailResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;
    let schema = state
        .usecase
        .get_knowledge_table(GetKnowledgeTableCommand {
            actor,
            organization_id,
            id: table_id,
        })
        .await?;
    Ok(Response::OK(schema.into()))
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}",
    operation_id = "updateKnowledgeTable",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("table_id" = mestier_core::KnowledgeTableId, Path, description = "Table identifier"),
    ),
    request_body = UpdateKnowledgeTableRequest,
    responses(
        (status = 200, description = "The table, renamed or redescribed; an absent field is kept, a null description is cleared", body = inline(DataEnvelope<KnowledgeTableResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Table not found, or held by another organization"),
        (status = 409, description = "A table already has this name"),
        (status = 422, description = "Empty or too long name or description"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update(
    TablePath {
        organization_id,
        table_id,
    }: TablePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateKnowledgeTableRequest>,
) -> Result<Response<KnowledgeTableResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let table = state
        .usecase
        .acting_as(user_id)
        .update_knowledge_table(UpdateKnowledgeTableCommand {
            actor,
            organization_id,
            id: table_id,
            name: payload.name,
            description: payload.description,
        })
        .await?
        .map_err(refused)?;
    Ok(Response::OK(table.into()))
}

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}",
    operation_id = "deleteKnowledgeTable",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("table_id" = mestier_core::KnowledgeTableId, Path, description = "Table identifier"),
    ),
    responses(
        (status = 204, description = "The table, its columns, views and rows are deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Table not found, or held by another organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete(
    TablePath {
        organization_id,
        table_id,
    }: TablePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<StatusCode, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    state
        .usecase
        .acting_as(user_id)
        .delete_knowledge_table(DeleteKnowledgeTableCommand {
            actor,
            organization_id,
            id: table_id,
        })
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, require_org_membership, resolve_actor};
use http::StatusCode;
use mestier_core::{
    ColumnConfig, ColumnKind, CreateTableColumnCommand, DeleteTableColumnCommand,
    ReorderTableColumnsCommand, TableColumnId, UpdateTableColumnCommand,
};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::tables::{
    ColumnOrderPath, ColumnPath, ColumnsPath, refused, response::TableColumnResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTableColumnRequest {
    pub name: String,
    pub kind: ColumnKind,
    #[serde(default)]
    pub config: ColumnConfig,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTableColumnRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub kind: Option<ColumnKind>,
    #[serde(default)]
    pub config: Option<ColumnConfig>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ReorderTableColumnsRequest {
    pub order: Vec<TableColumnId>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/columns",
    operation_id = "createTableColumn",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("table_id" = mestier_core::KnowledgeTableId, Path, description = "Table identifier"),
    ),
    request_body = CreateTableColumnRequest,
    responses(
        (status = 201, description = "The column, placed last. Options without an id receive one", body = inline(DataEnvelope<TableColumnResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Table not found, or held by another organization"),
        (status = 409, description = "A column of the table already has this name, whatever the case"),
        (status = 422, description = "Invalid name or config, or 50 columns already"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create(
    ColumnsPath {
        organization_id,
        table_id,
    }: ColumnsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateTableColumnRequest>,
) -> Result<Response<TableColumnResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let column = state
        .usecase
        .acting_as(user_id)
        .create_table_column(CreateTableColumnCommand {
            actor,
            organization_id,
            table_id,
            name: payload.name,
            kind: payload.kind,
            config: payload.config,
        })
        .await?
        .map_err(refused)?;
    Ok(Response::Created(column.into()))
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/columns/{column_id}",
    operation_id = "updateTableColumn",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("table_id" = mestier_core::KnowledgeTableId, Path, description = "Table identifier"),
        ("column_id" = mestier_core::TableColumnId, Path, description = "Column identifier"),
    ),
    request_body = UpdateTableColumnRequest,
    responses(
        (status = 200, description = "The column. A change of type rewrites the stored values; an option left out of the config is removed from every row", body = inline(DataEnvelope<TableColumnResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Table or column not found, or held by another organization"),
        (status = 409, description = "Name taken, primary column retyped, or a stored value that would not fit the new type"),
        (status = 422, description = "Invalid name or config, or a change of type that is not lossless"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update(
    ColumnPath {
        organization_id,
        table_id,
        column_id,
    }: ColumnPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateTableColumnRequest>,
) -> Result<Response<TableColumnResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let column = state
        .usecase
        .acting_as(user_id)
        .update_table_column(UpdateTableColumnCommand {
            actor,
            organization_id,
            table_id,
            id: column_id,
            name: payload.name,
            kind: payload.kind,
            config: payload.config,
        })
        .await?
        .map_err(refused)?;
    Ok(Response::OK(column.into()))
}

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/columns/{column_id}",
    operation_id = "deleteTableColumn",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("table_id" = mestier_core::KnowledgeTableId, Path, description = "Table identifier"),
        ("column_id" = mestier_core::TableColumnId, Path, description = "Column identifier"),
    ),
    responses(
        (status = 204, description = "The column and its value in every row are deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Table or column not found, or held by another organization"),
        (status = 409, description = "The primary column is never deleted"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete(
    ColumnPath {
        organization_id,
        table_id,
        column_id,
    }: ColumnPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<StatusCode, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    state
        .usecase
        .acting_as(user_id)
        .delete_table_column(DeleteTableColumnCommand {
            actor,
            organization_id,
            table_id,
            id: column_id,
        })
        .await?
        .map_err(refused)?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/columns/order",
    operation_id = "reorderTableColumns",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("table_id" = mestier_core::KnowledgeTableId, Path, description = "Table identifier"),
    ),
    request_body = ReorderTableColumnsRequest,
    responses(
        (status = 200, description = "The columns in their new order", body = inline(DataEnvelope<Vec<TableColumnResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Table not found, or held by another organization"),
        (status = 422, description = "The order does not list every column once, or the primary column is not first"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn reorder(
    ColumnOrderPath {
        organization_id,
        table_id,
    }: ColumnOrderPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<ReorderTableColumnsRequest>,
) -> Result<Response<Vec<TableColumnResponse>>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let columns = state
        .usecase
        .acting_as(user_id)
        .reorder_table_columns(ReorderTableColumnsCommand {
            actor,
            organization_id,
            table_id,
            order: payload.order,
        })
        .await?
        .map_err(refused)?;
    Ok(Response::OK(columns.into_iter().map(Into::into).collect()))
}

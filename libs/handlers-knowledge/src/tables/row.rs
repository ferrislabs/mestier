use std::collections::BTreeMap;

use auth::Identity;
use axum::{
    Extension, Json,
    extract::{Query, State},
};
use handlers::{ApiError, AppState, DataEnvelope, Response, require_org_membership, resolve_actor};
use http::StatusCode;
use mestier_core::{
    CreateTableRowCommand, DeleteTableRowsCommand, GetTableRowCommand, ListTableRowsCommand,
    MAX_TABLE_ROW_PAGE, RowFilter, RowSelection, RowSort, TableColumnId, TableRowId, TableViewId,
    UpdateTableRowCommand,
};
use serde::Deserialize;
use serde_json::Value;
use utoipa::{IntoParams, ToSchema};

use crate::tables::{
    RowDeletionPath, RowPath, RowQueryPath, RowsPath, refused,
    response::{DeletedTableRowsResponse, TableRowPageResponse, TableRowResponse},
};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListTableRowsParams {
    pub view_id: Option<TableViewId>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct QueryTableRowsRequest {
    #[serde(default)]
    pub filters: Vec<RowFilter>,
    #[serde(default)]
    pub sorts: Vec<RowSort>,
    #[serde(default)]
    pub limit: Option<u64>,
    #[serde(default)]
    pub offset: Option<u64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct WriteTableRowRequest {
    #[serde(default)]
    #[schema(value_type = Object)]
    pub values: BTreeMap<TableColumnId, Value>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DeleteTableRowsRequest {
    pub ids: Vec<TableRowId>,
}

async fn page(
    state: &AppState,
    identity: &Identity,
    path: RowsPath,
    selection: RowSelection,
    limit: Option<u64>,
    offset: Option<u64>,
) -> Result<Response<TableRowPageResponse>, ApiError> {
    require_org_membership(state, identity, path.organization_id).await?;
    let (_, actor) = resolve_actor(state, identity).await?;
    let page = state
        .usecase
        .list_table_rows(ListTableRowsCommand {
            actor,
            organization_id: path.organization_id,
            table_id: path.table_id,
            selection,
            limit: limit.unwrap_or(MAX_TABLE_ROW_PAGE),
            offset: offset.unwrap_or_default(),
        })
        .await?
        .map_err(refused)?;
    Ok(Response::OK(page.into()))
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/rows",
    operation_id = "listTableRows",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("table_id" = mestier_core::KnowledgeTableId, Path, description = "Table identifier"),
        ListTableRowsParams,
    ),
    responses(
        (status = 200, description = "A page of rows, filtered and sorted by the view when one is named, oldest first otherwise; at most 100 rows. Link, file and member cells carry their resolved label", body = inline(DataEnvelope<TableRowPageResponse>)),
        (status = 400, description = "A query parameter is malformed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Table or view not found, or held by another organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list(
    path: RowsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(params): Query<ListTableRowsParams>,
) -> Result<Response<TableRowPageResponse>, ApiError> {
    page(
        &state,
        &identity,
        path,
        RowSelection::View(params.view_id),
        params.limit,
        params.offset,
    )
    .await
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/rows/query",
    operation_id = "queryTableRows",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("table_id" = mestier_core::KnowledgeTableId, Path, description = "Table identifier"),
    ),
    request_body = QueryTableRowsRequest,
    responses(
        (status = 200, description = "A page of rows for ad hoc filters (all must hold) and up to 3 sorts, without saving a view", body = inline(DataEnvelope<TableRowPageResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Table not found, or held by another organization"),
        (status = 422, description = "Unknown column, operator that does not apply to the column, or value that does not fit it"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn query(
    RowQueryPath {
        organization_id,
        table_id,
    }: RowQueryPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<QueryTableRowsRequest>,
) -> Result<Response<TableRowPageResponse>, ApiError> {
    page(
        &state,
        &identity,
        RowsPath {
            organization_id,
            table_id,
        },
        RowSelection::AdHoc {
            filters: payload.filters,
            sorts: payload.sorts,
        },
        payload.limit,
        payload.offset,
    )
    .await
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/rows/{row_id}",
    operation_id = "getTableRow",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("table_id" = mestier_core::KnowledgeTableId, Path, description = "Table identifier"),
        ("row_id" = mestier_core::TableRowId, Path, description = "Row identifier"),
    ),
    responses(
        (status = 200, description = "The row, with resolved labels", body = inline(DataEnvelope<TableRowResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Table or row not found, or held by another organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get(
    RowPath {
        organization_id,
        table_id,
        row_id,
    }: RowPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<TableRowResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;
    let row = state
        .usecase
        .get_table_row(GetTableRowCommand {
            actor,
            organization_id,
            table_id,
            id: row_id,
        })
        .await?;
    Ok(Response::OK(row.into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/rows",
    operation_id = "createTableRow",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("table_id" = mestier_core::KnowledgeTableId, Path, description = "Table identifier"),
    ),
    request_body = WriteTableRowRequest,
    responses(
        (status = 201, description = "The row. `values` is keyed by column id; a null or absent value leaves the cell empty", body = inline(DataEnvelope<TableRowResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Table not found, or held by another organization"),
        (status = 422, description = "Unknown column, a value that does not fit its column, a link, file or member outside the organization, or 10000 rows already"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create(
    RowsPath {
        organization_id,
        table_id,
    }: RowsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<WriteTableRowRequest>,
) -> Result<Response<TableRowResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let row = state
        .usecase
        .acting_as(user_id)
        .create_table_row(CreateTableRowCommand {
            actor,
            organization_id,
            table_id,
            user_id: Some(user_id),
            values: payload.values,
        })
        .await?
        .map_err(refused)?;
    Ok(Response::Created(row.into()))
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/rows/{row_id}",
    operation_id = "updateTableRow",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("table_id" = mestier_core::KnowledgeTableId, Path, description = "Table identifier"),
        ("row_id" = mestier_core::TableRowId, Path, description = "Row identifier"),
    ),
    request_body = WriteTableRowRequest,
    responses(
        (status = 200, description = "The row. Only the cells sent change: they are merged into the stored values, a null clears its cell", body = inline(DataEnvelope<TableRowResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Table or row not found, or held by another organization"),
        (status = 422, description = "Unknown column, a value that does not fit its column, or a link, file or member outside the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update(
    RowPath {
        organization_id,
        table_id,
        row_id,
    }: RowPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<WriteTableRowRequest>,
) -> Result<Response<TableRowResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let row = state
        .usecase
        .acting_as(user_id)
        .update_table_row(UpdateTableRowCommand {
            actor,
            organization_id,
            table_id,
            id: row_id,
            user_id: Some(user_id),
            values: payload.values,
        })
        .await?
        .map_err(refused)?;
    Ok(Response::OK(row.into()))
}

async fn remove(
    state: &AppState,
    identity: &Identity,
    path: RowsPath,
    ids: Vec<TableRowId>,
) -> Result<u64, ApiError> {
    require_org_membership(state, identity, path.organization_id).await?;
    let (user_id, actor) = resolve_actor(state, identity).await?;
    state
        .usecase
        .acting_as(user_id)
        .delete_table_rows(DeleteTableRowsCommand {
            actor,
            organization_id: path.organization_id,
            table_id: path.table_id,
            ids,
        })
        .await?
        .map_err(refused)
}

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/rows/{row_id}",
    operation_id = "deleteTableRow",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("table_id" = mestier_core::KnowledgeTableId, Path, description = "Table identifier"),
        ("row_id" = mestier_core::TableRowId, Path, description = "Row identifier"),
    ),
    responses(
        (status = 204, description = "The row is deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Table or row not found, or held by another organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete(
    RowPath {
        organization_id,
        table_id,
        row_id,
    }: RowPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<StatusCode, ApiError> {
    let path = RowsPath {
        organization_id,
        table_id,
    };
    match remove(&state, &identity, path, vec![row_id]).await? {
        0 => Err(ApiError::NotFound),
        _ => Ok(StatusCode::NO_CONTENT),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/rows/delete",
    operation_id = "deleteTableRows",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("table_id" = mestier_core::KnowledgeTableId, Path, description = "Table identifier"),
    ),
    request_body = DeleteTableRowsRequest,
    responses(
        (status = 200, description = "How many of the rows were found in the table and deleted", body = inline(DataEnvelope<DeletedTableRowsResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Table not found, or held by another organization"),
        (status = 422, description = "More than 1000 rows at once"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_many(
    RowDeletionPath {
        organization_id,
        table_id,
    }: RowDeletionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<DeleteTableRowsRequest>,
) -> Result<Response<DeletedTableRowsResponse>, ApiError> {
    let path = RowsPath {
        organization_id,
        table_id,
    };
    let deleted = remove(&state, &identity, path, payload.ids).await?;
    Ok(Response::OK(DeletedTableRowsResponse { deleted }))
}

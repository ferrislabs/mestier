use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, require_org_membership, resolve_actor};
use http::StatusCode;
use mestier_core::{
    CreateTableViewCommand, DeleteTableViewCommand, UpdateTableViewCommand, ViewConfig, ViewKind,
};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::tables::{ViewPath, ViewsPath, refused, response::TableViewResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTableViewRequest {
    pub name: String,
    pub kind: ViewKind,
    #[serde(default)]
    pub config: ViewConfig,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTableViewRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub kind: Option<ViewKind>,
    #[serde(default)]
    pub config: Option<ViewConfig>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/views",
    operation_id = "createTableView",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("table_id" = mestier_core::KnowledgeTableId, Path, description = "Table identifier"),
    ),
    request_body = CreateTableViewRequest,
    responses(
        (status = 201, description = "The view, placed last", body = inline(DataEnvelope<TableViewResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Table not found, or held by another organization"),
        (status = 409, description = "A view of the table already has this name, whatever the case"),
        (status = 422, description = "Invalid name, filter, sort, width or grouping, or 20 views already"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create(
    ViewsPath {
        organization_id,
        table_id,
    }: ViewsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateTableViewRequest>,
) -> Result<Response<TableViewResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let view = state
        .usecase
        .acting_as(user_id)
        .create_table_view(CreateTableViewCommand {
            actor,
            organization_id,
            table_id,
            name: payload.name,
            kind: payload.kind,
            config: payload.config,
        })
        .await?
        .map_err(refused)?;
    Ok(Response::Created(view.into()))
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/views/{view_id}",
    operation_id = "updateTableView",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("table_id" = mestier_core::KnowledgeTableId, Path, description = "Table identifier"),
        ("view_id" = mestier_core::TableViewId, Path, description = "View identifier"),
    ),
    request_body = UpdateTableViewRequest,
    responses(
        (status = 200, description = "The view. A config replaces the whole stored config", body = inline(DataEnvelope<TableViewResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Table or view not found, or held by another organization"),
        (status = 409, description = "A view of the table already has this name"),
        (status = 422, description = "Invalid name, filter, sort, width or grouping"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update(
    ViewPath {
        organization_id,
        table_id,
        view_id,
    }: ViewPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateTableViewRequest>,
) -> Result<Response<TableViewResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let view = state
        .usecase
        .acting_as(user_id)
        .update_table_view(UpdateTableViewCommand {
            actor,
            organization_id,
            table_id,
            id: view_id,
            name: payload.name,
            kind: payload.kind,
            config: payload.config,
        })
        .await?
        .map_err(refused)?;
    Ok(Response::OK(view.into()))
}

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/views/{view_id}",
    operation_id = "deleteTableView",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("table_id" = mestier_core::KnowledgeTableId, Path, description = "Table identifier"),
        ("view_id" = mestier_core::TableViewId, Path, description = "View identifier"),
    ),
    responses(
        (status = 204, description = "The view is deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Table or view not found, or held by another organization"),
        (status = 409, description = "A table keeps at least one view"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete(
    ViewPath {
        organization_id,
        table_id,
        view_id,
    }: ViewPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<StatusCode, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    state
        .usecase
        .acting_as(user_id)
        .delete_table_view(DeleteTableViewCommand {
            actor,
            organization_id,
            table_id,
            id: view_id,
        })
        .await?
        .map_err(refused)?;
    Ok(StatusCode::NO_CONTENT)
}

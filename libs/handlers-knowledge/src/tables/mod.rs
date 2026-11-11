use axum::{Router, extract::DefaultBodyLimit};
use axum_extra::routing::{RouterExt, TypedPath};
use handlers::{ApiError, AppState};
use mestier_core::{
    KnowledgeTableId, OrganizationId, TableColumnId, TableError, TableRowId, TableViewId,
};
use serde::Deserialize;

pub mod column;
pub mod response;
pub mod row;
pub mod table;
pub mod view;

pub const MAX_BODY_BYTES: usize = 256 * 1024;

pub fn routes() -> Router<AppState> {
    Router::new()
        .typed_get(table::list)
        .typed_post(table::create)
        .typed_get(table::get)
        .typed_patch(table::update)
        .typed_delete(table::delete)
        .typed_post(column::create)
        .typed_put(column::reorder)
        .typed_patch(column::update)
        .typed_delete(column::delete)
        .typed_post(view::create)
        .typed_patch(view::update)
        .typed_delete(view::delete)
        .typed_get(row::list)
        .typed_post(row::create)
        .typed_post(row::query)
        .typed_post(row::delete_many)
        .typed_get(row::get)
        .typed_patch(row::update)
        .typed_delete(row::delete)
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/knowledge/tables")]
pub struct TablesPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}")]
pub struct TablePath {
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/columns")]
pub struct ColumnsPath {
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/columns/order")]
pub struct ColumnOrderPath {
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/columns/{column_id}"
)]
pub struct ColumnPath {
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
    pub column_id: TableColumnId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/views")]
pub struct ViewsPath {
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/views/{view_id}")]
pub struct ViewPath {
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
    pub view_id: TableViewId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/rows")]
pub struct RowsPath {
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/rows/query")]
pub struct RowQueryPath {
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/rows/delete")]
pub struct RowDeletionPath {
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/rows/{row_id}")]
pub struct RowPath {
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
    pub row_id: TableRowId,
}

pub(crate) fn refused(error: TableError) -> ApiError {
    if error.is_conflict() {
        ApiError::Conflict(error.to_string())
    } else {
        ApiError::UnprocessableEntity(error.to_string())
    }
}

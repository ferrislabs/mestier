use std::collections::BTreeMap;

use bytes::Bytes;
use common::UserId;
use serde_json::Value;

use crate::{
    OrganizationId,
    domain::knowledge::tables::{
        KnowledgeTableId, RowFilter, RowSort, TableColumnId, TableRowId, TableViewId, ViewConfig,
        ViewKind,
        column::{ColumnConfig, ColumnKind},
    },
};

#[derive(Debug, Clone)]
pub struct CreateKnowledgeTableCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub user_id: Option<UserId>,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ListKnowledgeTablesCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
}

#[derive(Debug, Clone)]
pub struct GetKnowledgeTableCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: KnowledgeTableId,
}

#[derive(Debug, Clone)]
pub struct UpdateKnowledgeTableCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: KnowledgeTableId,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
}

#[derive(Debug, Clone)]
pub struct DeleteKnowledgeTableCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: KnowledgeTableId,
}

#[derive(Debug, Clone)]
pub struct CreateTableColumnCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
    pub name: String,
    pub kind: ColumnKind,
    pub config: ColumnConfig,
}

#[derive(Debug, Clone)]
pub struct UpdateTableColumnCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
    pub id: TableColumnId,
    pub name: Option<String>,
    pub kind: Option<ColumnKind>,
    pub config: Option<ColumnConfig>,
}

#[derive(Debug, Clone)]
pub struct DeleteTableColumnCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
    pub id: TableColumnId,
}

#[derive(Debug, Clone)]
pub struct ReorderTableColumnsCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
    pub order: Vec<TableColumnId>,
}

#[derive(Debug, Clone)]
pub struct CreateTableViewCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
    pub name: String,
    pub kind: ViewKind,
    pub config: ViewConfig,
}

#[derive(Debug, Clone)]
pub struct UpdateTableViewCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
    pub id: TableViewId,
    pub name: Option<String>,
    pub kind: Option<ViewKind>,
    pub config: Option<ViewConfig>,
}

#[derive(Debug, Clone)]
pub struct DeleteTableViewCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
    pub id: TableViewId,
}

#[derive(Debug, Clone)]
pub enum RowSelection {
    View(Option<TableViewId>),
    AdHoc {
        filters: Vec<RowFilter>,
        sorts: Vec<RowSort>,
    },
}

#[derive(Debug, Clone)]
pub struct ListTableRowsCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
    pub selection: RowSelection,
    pub limit: u64,
    pub offset: u64,
}

#[derive(Debug, Clone)]
pub struct GetTableRowCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
    pub id: TableRowId,
}

#[derive(Debug, Clone)]
pub struct CreateTableRowCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
    pub user_id: Option<UserId>,
    pub values: BTreeMap<TableColumnId, Value>,
}

#[derive(Debug, Clone)]
pub struct UpdateTableRowCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
    pub id: TableRowId,
    pub user_id: Option<UserId>,
    pub values: BTreeMap<TableColumnId, Value>,
}

#[derive(Debug, Clone)]
pub struct DeleteTableRowsCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
    pub ids: Vec<TableRowId>,
}

#[derive(Debug, Clone)]
pub struct ImportKnowledgeTableCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub user_id: Option<UserId>,
    pub name: String,
    pub file: Bytes,
}

#[derive(Debug, Clone)]
pub struct ImportTableRowsCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
    pub user_id: Option<UserId>,
    pub file: Bytes,
}

#[derive(Debug, Clone)]
pub struct ExportTableViewCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
    pub view_id: TableViewId,
}

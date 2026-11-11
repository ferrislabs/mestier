use std::collections::HashMap;

use common::CoreError;
use serde_json::Value;

use crate::{
    OrganizationId,
    domain::knowledge::tables::{
        KnowledgeTable, KnowledgeTableId, RowBatch, RowQuery, RowUpdate, SelectOptionId,
        TableColumn, TableColumnId, TableRow, TableRowId, TableView, TableViewId,
        column::ColumnKind, value::CellReference,
    },
};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait KnowledgeTableRepository: Send {
    fn lock_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn lock_table(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeTableId,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn count_tables(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<u64, CoreError>> + Send;

    fn insert_table(
        &mut self,
        table: &KnowledgeTable,
    ) -> impl Future<Output = Result<KnowledgeTable, CoreError>> + Send;

    fn find_table(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeTableId,
    ) -> impl Future<Output = Result<Option<KnowledgeTable>, CoreError>> + Send;

    fn list_tables(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<KnowledgeTable>, CoreError>> + Send;

    fn update_table(
        &mut self,
        table: &KnowledgeTable,
    ) -> impl Future<Output = Result<KnowledgeTable, CoreError>> + Send;

    fn delete_table(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeTableId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn insert_column(
        &mut self,
        column: &TableColumn,
    ) -> impl Future<Output = Result<TableColumn, CoreError>> + Send;

    fn columns(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
    ) -> impl Future<Output = Result<Vec<TableColumn>, CoreError>> + Send;

    fn update_column(
        &mut self,
        column: &TableColumn,
    ) -> impl Future<Output = Result<TableColumn, CoreError>> + Send;

    fn delete_column(
        &mut self,
        organization_id: OrganizationId,
        id: TableColumnId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn reorder_columns(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
        order: &[TableColumnId],
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn insert_view(
        &mut self,
        view: &TableView,
    ) -> impl Future<Output = Result<TableView, CoreError>> + Send;

    fn views(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
    ) -> impl Future<Output = Result<Vec<TableView>, CoreError>> + Send;

    fn update_view(
        &mut self,
        view: &TableView,
    ) -> impl Future<Output = Result<TableView, CoreError>> + Send;

    fn delete_view(
        &mut self,
        organization_id: OrganizationId,
        id: TableViewId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait KnowledgeTableRowRepository: Send {
    fn count(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
    ) -> impl Future<Output = Result<u64, CoreError>> + Send;

    fn insert(
        &mut self,
        row: &TableRow,
    ) -> impl Future<Output = Result<TableRow, CoreError>> + Send;

    fn insert_many(
        &mut self,
        batch: &RowBatch,
    ) -> impl Future<Output = Result<u64, CoreError>> + Send;

    fn find(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
        id: TableRowId,
    ) -> impl Future<Output = Result<Option<TableRow>, CoreError>> + Send;

    fn patch(
        &mut self,
        update: &RowUpdate,
    ) -> impl Future<Output = Result<Option<TableRow>, CoreError>> + Send;

    fn delete(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
        ids: &[TableRowId],
    ) -> impl Future<Output = Result<u64, CoreError>> + Send;

    fn list(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
        query: &RowQuery,
    ) -> impl Future<Output = Result<(Vec<TableRow>, u64), CoreError>> + Send;

    fn remove_column(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
        column: TableColumnId,
    ) -> impl Future<Output = Result<u64, CoreError>> + Send;

    fn remove_option(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
        column: TableColumnId,
        kind: ColumnKind,
        option: SelectOptionId,
    ) -> impl Future<Output = Result<u64, CoreError>> + Send;

    fn values_of(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
        column: TableColumnId,
    ) -> impl Future<Output = Result<Vec<(TableRowId, Value)>, CoreError>> + Send;

    fn rewrite_values(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
        column: TableColumnId,
        values: &[(TableRowId, Value)],
    ) -> impl Future<Output = Result<u64, CoreError>> + Send;

    fn labels(
        &mut self,
        organization_id: OrganizationId,
        references: &[CellReference],
    ) -> impl Future<Output = Result<HashMap<CellReference, String>, CoreError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait CellReferenceCheck: Send {
    fn exists(
        &mut self,
        organization_id: OrganizationId,
        reference: CellReference,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}

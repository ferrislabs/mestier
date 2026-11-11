use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use common::UserId;
use mestier_core::{
    CellLabel, ColumnConfig, ColumnKind, ImportedTable, InvalidCsvCell, KnowledgeTable,
    KnowledgeTableId, ResolvedRow, RowPage, TableColumn, TableColumnId, TableImportReport,
    TableRowId, TableSchema, TableView, TableViewId, ViewConfig, ViewKind,
};
use serde::Serialize;
use serde_json::{Map, Value};
use utoipa::ToSchema;

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct KnowledgeTableResponse {
    pub id: KnowledgeTableId,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<KnowledgeTable> for KnowledgeTableResponse {
    fn from(table: KnowledgeTable) -> Self {
        Self {
            id: table.id,
            name: table.name.into_inner(),
            description: table.description.map(|text| text.as_str().to_owned()),
            created_at: table.created_at,
            updated_at: table.updated_at,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct TableColumnResponse {
    pub id: TableColumnId,
    pub name: String,
    pub kind: ColumnKind,
    pub config: ColumnConfig,
    pub position: i32,
    pub is_primary: bool,
}

impl From<TableColumn> for TableColumnResponse {
    fn from(column: TableColumn) -> Self {
        Self {
            id: column.id,
            kind: column.kind(),
            config: column.column_type.config(),
            name: column.name.into_inner(),
            position: column.position,
            is_primary: column.is_primary,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct TableViewResponse {
    pub id: TableViewId,
    pub name: String,
    pub kind: ViewKind,
    pub config: ViewConfig,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<TableView> for TableViewResponse {
    fn from(view: TableView) -> Self {
        Self {
            id: view.id,
            name: view.name.into_inner(),
            kind: view.kind,
            config: view.config,
            position: view.position,
            created_at: view.created_at,
            updated_at: view.updated_at,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct KnowledgeTableDetailResponse {
    pub id: KnowledgeTableId,
    pub name: String,
    pub description: Option<String>,
    pub columns: Vec<TableColumnResponse>,
    pub views: Vec<TableViewResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<TableSchema> for KnowledgeTableDetailResponse {
    fn from(schema: TableSchema) -> Self {
        let table = KnowledgeTableResponse::from(schema.table);
        Self {
            id: table.id,
            name: table.name,
            description: table.description,
            columns: schema.columns.into_iter().map(Into::into).collect(),
            views: schema.views.into_iter().map(Into::into).collect(),
            created_at: table.created_at,
            updated_at: table.updated_at,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct TableCellLabelResponse {
    pub label: Option<String>,
    pub missing: bool,
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct TableRowResponse {
    pub id: TableRowId,
    #[schema(value_type = Object)]
    pub values: Map<String, Value>,
    pub labels: BTreeMap<String, TableCellLabelResponse>,
    pub created_by: Option<UserId>,
    pub updated_by: Option<UserId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ResolvedRow> for TableRowResponse {
    fn from(resolved: ResolvedRow) -> Self {
        let row = resolved.row;
        Self {
            id: row.id,
            values: row.values,
            labels: resolved
                .labels
                .into_iter()
                .map(|(column, label)| {
                    let response = match label {
                        CellLabel::Found(label) => TableCellLabelResponse {
                            label: Some(label),
                            missing: false,
                        },
                        CellLabel::Missing => TableCellLabelResponse {
                            label: None,
                            missing: true,
                        },
                    };
                    (column.to_string(), response)
                })
                .collect(),
            created_by: row.created_by,
            updated_by: row.updated_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct TableRowPageResponse {
    pub rows: Vec<TableRowResponse>,
    pub total: u64,
    pub limit: u64,
    pub offset: u64,
}

impl From<RowPage> for TableRowPageResponse {
    fn from(page: RowPage) -> Self {
        Self {
            rows: page.rows.into_iter().map(Into::into).collect(),
            total: page.total,
            limit: page.limit,
            offset: page.offset,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct DeletedTableRowsResponse {
    pub deleted: u64,
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct InvalidCsvCellResponse {
    pub row: u64,
    pub column: String,
    pub value: String,
    pub reason: String,
    pub message: String,
}

impl From<InvalidCsvCell> for InvalidCsvCellResponse {
    fn from(cell: InvalidCsvCell) -> Self {
        Self {
            row: cell.row,
            column: cell.column,
            value: cell.value,
            reason: cell.reason.code().to_owned(),
            message: cell.reason.to_string(),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct TableImportReportResponse {
    pub rows_created: u64,
    pub ignored_columns: Vec<String>,
    pub invalid_cells: Vec<InvalidCsvCellResponse>,
    pub invalid_cell_count: u64,
}

impl From<TableImportReport> for TableImportReportResponse {
    fn from(report: TableImportReport) -> Self {
        Self {
            rows_created: report.rows_created,
            ignored_columns: report.ignored_columns,
            invalid_cells: report.invalid_cells.into_iter().map(Into::into).collect(),
            invalid_cell_count: report.invalid_cell_count,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct ImportedKnowledgeTableResponse {
    pub table: KnowledgeTableDetailResponse,
    pub report: TableImportReportResponse,
}

impl From<ImportedTable> for ImportedKnowledgeTableResponse {
    fn from(imported: ImportedTable) -> Self {
        Self {
            table: imported.schema.into(),
            report: imported.report.into(),
        }
    }
}

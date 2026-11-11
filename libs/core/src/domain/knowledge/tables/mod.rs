use std::{collections::BTreeMap, fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use common::UserId;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::OrganizationId;

pub mod column;
pub mod commands;
pub mod csv;
pub mod error;
pub mod events;
pub mod filter;
pub mod import;
pub mod name;
pub mod ports;
pub mod rows;
pub mod service;
pub mod value;
pub mod view;

pub use column::{
    ColumnConfig, ColumnKind, ColumnType, Decimals, SelectColor, SelectOption, SelectOptionInput,
    SelectOptions,
};
pub use commands::*;
pub use error::{TableError, TableOutcome};
pub use filter::{FilterOperator, RowCondition, RowFilter, RowQuery, RowSort, SortDirection};
pub use name::{TableDescription, TableItemName};
pub use value::{CellError, CellReference, CellValue, MISSING_LABEL, RowPatch, RowValues};
pub use view::{ViewConfig, ViewKind};

macro_rules! table_id {
    ($name:ident) => {
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize,
            ToSchema,
        )]
        pub struct $name(pub Uuid);

        impl FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Uuid::from_str(s).map($name)
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

table_id!(KnowledgeTableId);
table_id!(TableColumnId);
table_id!(TableViewId);
table_id!(TableRowId);
table_id!(SelectOptionId);

pub const MAX_TABLES: u64 = 200;
pub const MAX_COLUMNS: usize = 50;
pub const MAX_ROWS: u64 = 10_000;
pub const MAX_OPTIONS: usize = 100;
pub const MAX_VIEWS: usize = 20;
pub const MAX_ROW_PAGE: u64 = 100;
pub const MAX_DELETED_ROWS: usize = 1_000;
pub const PRIMARY_COLUMN_NAME: &str = "Nom";
pub const DEFAULT_VIEW_NAME: &str = "Grille";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeTable {
    pub id: KnowledgeTableId,
    pub organization_id: OrganizationId,
    pub name: TableItemName,
    pub description: Option<TableDescription>,
    pub created_by: Option<UserId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableColumn {
    pub id: TableColumnId,
    pub table_id: KnowledgeTableId,
    pub organization_id: OrganizationId,
    pub name: TableItemName,
    pub column_type: ColumnType,
    pub position: i32,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
}

pub fn column_named<'a>(columns: &'a [TableColumn], name: &str) -> Option<&'a TableColumn> {
    columns.iter().find(|column| column.name.same_as(name))
}

impl TableColumn {
    pub fn kind(&self) -> ColumnKind {
        self.column_type.kind()
    }

    pub fn key(&self) -> String {
        self.id.to_string()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableView {
    pub id: TableViewId,
    pub table_id: KnowledgeTableId,
    pub organization_id: OrganizationId,
    pub name: TableItemName,
    pub kind: ViewKind,
    pub config: ViewConfig,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableRow {
    pub id: TableRowId,
    pub table_id: KnowledgeTableId,
    pub organization_id: OrganizationId,
    pub values: Map<String, Value>,
    pub created_by: Option<UserId>,
    pub updated_by: Option<UserId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TableRow {
    pub fn title<'a>(&'a self, columns: &[TableColumn]) -> Option<&'a str> {
        let primary = columns.iter().find(|column| column.is_primary)?;
        self.values.get(&primary.key()).and_then(Value::as_str)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableSchema {
    pub table: KnowledgeTable,
    pub columns: Vec<TableColumn>,
    pub views: Vec<TableView>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CellLabel {
    Found(String),
    Missing,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedRow {
    pub row: TableRow,
    pub labels: BTreeMap<TableColumnId, CellLabel>,
}

impl ResolvedRow {
    pub fn shown(&self, column: &TableColumn) -> Option<String> {
        let raw = self.row.values.get(&column.key())?;
        match self.labels.get(&column.id) {
            Some(CellLabel::Found(label)) => Some(label.clone()),
            Some(CellLabel::Missing) => Some(MISSING_LABEL.to_owned()),
            None => CellValue::parse(&column.column_type, raw)
                .ok()
                .flatten()
                .map(|cell| cell.display(&column.column_type, &Default::default())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RowPage {
    pub rows: Vec<ResolvedRow>,
    pub total: u64,
    pub limit: u64,
    pub offset: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RowBatch {
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
    pub created_by: Option<UserId>,
    pub created_at: DateTime<Utc>,
    pub rows: Vec<(TableRowId, RowValues)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RowUpdate {
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
    pub id: TableRowId,
    pub set: Map<String, Value>,
    pub cleared: Vec<String>,
    pub updated_by: Option<UserId>,
    pub updated_at: DateTime<Utc>,
}

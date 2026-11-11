use common::CoreError;

use crate::domain::knowledge::tables::{TableColumnId, value::CellError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TableError {
    #[error("name: a name cannot be empty")]
    NameEmpty,
    #[error("name: a name holds at most 100 characters")]
    NameTooLong,
    #[error("description: a description holds at most 2000 characters")]
    DescriptionTooLong,
    #[error("name: a table with this name already exists")]
    TableNameTaken,
    #[error("name: a column with this name already exists in the table")]
    ColumnNameTaken,
    #[error("name: a view with this name already exists in the table")]
    ViewNameTaken,
    #[error("tables: an organization holds at most 200 tables")]
    TooManyTables,
    #[error("columns: a table holds at most 50 columns")]
    TooManyColumns,
    #[error("rows: a table holds at most 10000 rows")]
    TooManyRows,
    #[error("views: a table holds at most 20 views")]
    TooManyViews,
    #[error("options: a choice column holds at most 100 options")]
    TooManyOptions,
    #[error("options: an option label cannot be empty")]
    OptionLabelEmpty,
    #[error("options: an option label holds at most 100 characters")]
    OptionLabelTooLong,
    #[error("options: two options share a label")]
    OptionLabelTaken,
    #[error("options: two options share an id")]
    OptionIdTaken,
    #[error("decimals: a number shows 0 to 4 decimals")]
    DecimalsOutOfRange,
    #[error("link_kind: a link column names customer, project or quote")]
    LinkKindMissing,
    #[error("column: the primary column is never deleted and stays text")]
    PrimaryColumnLocked,
    #[error("columns: the primary column stays first")]
    PrimaryColumnFirst,
    #[error("columns: the order lists every column of the table once")]
    ColumnOrderMismatch,
    #[error("kind: this change of type would lose values; create a new column instead")]
    ConversionRefused,
    #[error("kind: a stored value does not fit the new type; nothing was changed")]
    ConversionLosesValue,
    #[error("views: a table keeps at least one view")]
    LastView,
    #[error("values.{0}: no such column in the table")]
    UnknownColumn(TableColumnId),
    #[error("values.{column}: {reason}")]
    InvalidCell {
        column: TableColumnId,
        reason: CellError,
    },
    #[error("filters.{0}: this operator does not apply to the column")]
    InvalidFilter(TableColumnId),
    #[error("filters.{0}: the value does not fit the column")]
    InvalidFilterValue(TableColumnId),
    #[error("filters: a view holds at most 20 filters")]
    TooManyFilters,
    #[error("sorts.{0}: link, file and member columns cannot be sorted")]
    UnsortableColumn(TableColumnId),
    #[error("sorts: a view sorts on at most 3 columns")]
    TooManySorts,
    #[error("widths: a column is 40 to 1000 pixels wide")]
    WidthOutOfRange,
    #[error("group_by: a kanban groups by a single choice column")]
    KanbanNeedsSingleSelect,
    #[error("ids: at most 1000 rows are deleted at once")]
    TooManyRowsAtOnce,
    #[error("file: a CSV file holds at most 5 MB")]
    CsvTooLarge,
    #[error("file: a CSV file holds at most 10000 rows")]
    CsvTooManyRows,
    #[error("file: a CSV file becomes a table of at most 50 columns")]
    CsvTooManyColumns,
    #[error("file: the CSV file has no header line")]
    CsvEmpty,
    #[error("file: the file cannot be read as CSV")]
    CsvUnreadable,
    #[error("file: no column of the file matches a column of the table")]
    CsvNoKnownColumn,
}

impl TableError {
    pub fn is_conflict(self) -> bool {
        matches!(
            self,
            Self::TableNameTaken
                | Self::ColumnNameTaken
                | Self::ViewNameTaken
                | Self::LastView
                | Self::PrimaryColumnLocked
                | Self::ConversionLosesValue
        )
    }
}

pub type TableOutcome<T> = Result<T, TableError>;

#[derive(Debug)]
pub(crate) enum Stop {
    Refused(TableError),
    Failed(CoreError),
}

impl From<TableError> for Stop {
    fn from(error: TableError) -> Self {
        Stop::Refused(error)
    }
}

impl From<CoreError> for Stop {
    fn from(error: CoreError) -> Self {
        Stop::Failed(error)
    }
}

pub(crate) fn settle<T>(result: Result<T, Stop>) -> Result<TableOutcome<T>, CoreError> {
    match result {
        Ok(value) => Ok(Ok(value)),
        Err(Stop::Refused(error)) => Ok(Err(error)),
        Err(Stop::Failed(error)) => Err(error),
    }
}

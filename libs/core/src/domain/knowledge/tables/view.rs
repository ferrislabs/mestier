use std::{collections::BTreeMap, str::FromStr};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::knowledge::tables::{
    TableColumn, TableColumnId, TableError,
    column::ColumnType,
    filter::{MAX_FILTERS, MAX_SORTS, RowFilter, RowSort},
};

pub const MIN_WIDTH: u16 = 40;
pub const MAX_WIDTH: u16 = 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ViewKind {
    Grid,
    Kanban,
}

impl ViewKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ViewKind::Grid => "grid",
            ViewKind::Kanban => "kanban",
        }
    }
}

impl FromStr for ViewKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "grid" => Ok(ViewKind::Grid),
            "kanban" => Ok(ViewKind::Kanban),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ViewConfig {
    #[serde(default)]
    pub filters: Vec<RowFilter>,
    #[serde(default)]
    pub sorts: Vec<RowSort>,
    #[serde(default)]
    pub hidden: Vec<TableColumnId>,
    #[serde(default)]
    #[schema(value_type = BTreeMap<String, u16>)]
    pub widths: BTreeMap<TableColumnId, u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_by: Option<TableColumnId>,
}

fn exists(columns: &[TableColumn], id: TableColumnId) -> bool {
    columns.iter().any(|column| column.id == id)
}

fn groups(columns: &[TableColumn], id: TableColumnId) -> bool {
    columns
        .iter()
        .any(|column| column.id == id && matches!(column.column_type, ColumnType::SingleSelect(_)))
}

impl ViewConfig {
    pub fn check(&self, kind: ViewKind, columns: &[TableColumn]) -> Result<Self, TableError> {
        if self.filters.len() > MAX_FILTERS {
            return Err(TableError::TooManyFilters);
        }
        if self.sorts.len() > MAX_SORTS {
            return Err(TableError::TooManySorts);
        }
        for filter in &self.filters {
            filter.compile(columns)?;
        }
        for sort in &self.sorts {
            sort.compile(columns)?;
        }
        let mut hidden: Vec<TableColumnId> = Vec::with_capacity(self.hidden.len());
        for id in &self.hidden {
            if !exists(columns, *id) {
                return Err(TableError::UnknownColumn(*id));
            }
            if !hidden.contains(id) {
                hidden.push(*id);
            }
        }
        for (id, width) in &self.widths {
            if !exists(columns, *id) {
                return Err(TableError::UnknownColumn(*id));
            }
            if !(MIN_WIDTH..=MAX_WIDTH).contains(width) {
                return Err(TableError::WidthOutOfRange);
            }
        }
        let group_by = match kind {
            ViewKind::Grid => None,
            ViewKind::Kanban => Some(
                self.group_by
                    .filter(|id| groups(columns, *id))
                    .ok_or(TableError::KanbanNeedsSingleSelect)?,
            ),
        };
        Ok(Self {
            filters: self.filters.clone(),
            sorts: self.sorts.clone(),
            hidden,
            widths: self.widths.clone(),
            group_by,
        })
    }

    pub fn reconcile(&self, kind: ViewKind, columns: &[TableColumn]) -> (ViewKind, Self) {
        let group_by = self.group_by.filter(|id| groups(columns, *id));
        let kind = match (kind, group_by) {
            (ViewKind::Kanban, None) => ViewKind::Grid,
            (kind, _) => kind,
        };
        let config = Self {
            filters: self
                .filters
                .iter()
                .filter(|filter| filter.compile(columns).is_ok())
                .cloned()
                .collect(),
            sorts: self
                .sorts
                .iter()
                .filter(|sort| sort.compile(columns).is_ok())
                .copied()
                .collect(),
            hidden: self
                .hidden
                .iter()
                .filter(|id| exists(columns, **id))
                .copied()
                .collect(),
            widths: self
                .widths
                .iter()
                .filter(|(id, _)| exists(columns, **id))
                .map(|(id, width)| (*id, *width))
                .collect(),
            group_by: (kind == ViewKind::Kanban).then_some(group_by).flatten(),
        };
        (kind, config)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use common::generate_uuid_v7;
    use serde_json::json;

    use super::*;
    use crate::{
        OrganizationId,
        domain::knowledge::tables::{
            FilterOperator, KnowledgeTableId, SelectOptionInput, SelectOptions, SortDirection,
            TableItemName,
        },
    };

    fn column(column_type: ColumnType) -> TableColumn {
        TableColumn {
            id: TableColumnId(generate_uuid_v7()),
            table_id: KnowledgeTableId(generate_uuid_v7()),
            organization_id: OrganizationId(generate_uuid_v7()),
            name: TableItemName::try_from("Colonne").unwrap(),
            column_type,
            position: 0,
            is_primary: false,
            created_at: Utc::now(),
        }
    }

    fn status() -> TableColumn {
        column(ColumnType::SingleSelect(
            SelectOptions::parse(&[SelectOptionInput {
                id: None,
                label: "En cours".to_owned(),
                color: None,
            }])
            .unwrap(),
        ))
    }

    #[test]
    fn a_kanban_groups_by_a_single_choice() {
        let name = column(ColumnType::Text);
        let status = status();
        let columns = [name.clone(), status.clone()];
        let grouped = |id| ViewConfig {
            group_by: Some(id),
            ..ViewConfig::default()
        };
        assert_eq!(
            grouped(status.id)
                .check(ViewKind::Kanban, &columns)
                .unwrap()
                .group_by,
            Some(status.id)
        );
        assert_eq!(
            grouped(name.id).check(ViewKind::Kanban, &columns),
            Err(TableError::KanbanNeedsSingleSelect)
        );
        assert_eq!(
            ViewConfig::default().check(ViewKind::Kanban, &columns),
            Err(TableError::KanbanNeedsSingleSelect)
        );
        assert_eq!(
            grouped(status.id)
                .check(ViewKind::Grid, &columns)
                .unwrap()
                .group_by,
            None
        );
    }

    #[test]
    fn a_view_refers_only_to_columns_of_its_table() {
        let name = column(ColumnType::Text);
        let stranger = TableColumnId(generate_uuid_v7());
        let config = ViewConfig {
            hidden: vec![stranger],
            ..ViewConfig::default()
        };
        assert_eq!(
            config.check(ViewKind::Grid, std::slice::from_ref(&name)),
            Err(TableError::UnknownColumn(stranger))
        );
        let narrow = ViewConfig {
            widths: BTreeMap::from([(name.id, 10)]),
            ..ViewConfig::default()
        };
        assert_eq!(
            narrow.check(ViewKind::Grid, std::slice::from_ref(&name)),
            Err(TableError::WidthOutOfRange)
        );
    }

    #[test]
    fn a_removed_column_leaves_the_view_and_a_lost_group_falls_back_to_grid() {
        let name = column(ColumnType::Text);
        let status = status();
        let config = ViewConfig {
            filters: vec![RowFilter {
                column_id: name.id,
                operator: FilterOperator::Contains,
                value: Some(json!("villa")),
            }],
            sorts: vec![RowSort {
                column_id: status.id,
                direction: SortDirection::Asc,
            }],
            hidden: vec![status.id],
            widths: BTreeMap::from([(status.id, 200)]),
            group_by: Some(status.id),
        };
        let (kind, kept) = config.reconcile(ViewKind::Kanban, std::slice::from_ref(&name));
        assert_eq!(kind, ViewKind::Grid);
        assert_eq!(kept.filters, config.filters);
        assert!(kept.sorts.is_empty() && kept.hidden.is_empty() && kept.widths.is_empty());
        assert_eq!(kept.group_by, None);
    }
}

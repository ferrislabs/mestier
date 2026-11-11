use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use crate::domain::knowledge::tables::{
    MAX_OPTIONS, MAX_ROW_PAGE, MAX_ROWS, TableColumn, TableColumnId, TableError,
    column::ColumnType, value::CellValue,
};

pub const MAX_FILTERS: usize = 20;
pub const MAX_SORTS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    Contains,
    Is,
    IsNot,
    AnyOf,
    AllOf,
    IsEmpty,
    IsNotEmpty,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Before,
    After,
    On,
    Checked,
    Unchecked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct RowFilter {
    pub column_id: TableColumnId,
    pub operator: FilterOperator,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<Object>)]
    pub value: Option<Value>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct RowSort {
    pub column_id: TableColumnId,
    #[serde(default)]
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparison {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowCondition {
    Empty,
    NotEmpty,
    TextContains(String),
    TextIs(String),
    NumberCompare(Comparison, String),
    DateCompare(Comparison, NaiveDate),
    Checked(bool),
    OptionIs(String),
    OptionIsNot(String),
    OptionAnyOf(Vec<String>),
    ListAnyOf(Vec<String>),
    ListAllOf(Vec<String>),
    LinkIs(String),
    IdIs(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledFilter {
    pub column: TableColumnId,
    pub condition: RowCondition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortKey {
    Text,
    Number,
    Date,
    Checkbox,
    OptionRank(Vec<String>),
    FirstOptionRank(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledSort {
    pub column: TableColumnId,
    pub key: SortKey,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowQuery {
    pub filters: Vec<CompiledFilter>,
    pub sorts: Vec<CompiledSort>,
    pub limit: u64,
    pub offset: u64,
}

fn column_of(columns: &[TableColumn], id: TableColumnId) -> Result<&TableColumn, TableError> {
    columns
        .iter()
        .find(|column| column.id == id)
        .ok_or(TableError::UnknownColumn(id))
}

fn allowed(column_type: &ColumnType, operator: FilterOperator) -> bool {
    use FilterOperator as Op;
    match column_type {
        ColumnType::Text | ColumnType::LongText => {
            matches!(
                operator,
                Op::Contains | Op::Is | Op::IsEmpty | Op::IsNotEmpty
            )
        }
        ColumnType::Number(_) | ColumnType::Currency => matches!(
            operator,
            Op::Eq | Op::Ne | Op::Lt | Op::Le | Op::Gt | Op::Ge | Op::IsEmpty | Op::IsNotEmpty
        ),
        ColumnType::Date => matches!(
            operator,
            Op::Before | Op::After | Op::On | Op::IsEmpty | Op::IsNotEmpty
        ),
        ColumnType::Checkbox => matches!(operator, Op::Checked | Op::Unchecked),
        ColumnType::SingleSelect(_) => {
            matches!(operator, Op::Is | Op::IsNot | Op::AnyOf | Op::IsEmpty)
        }
        ColumnType::MultiSelect(_) => matches!(operator, Op::AnyOf | Op::AllOf | Op::IsEmpty),
        ColumnType::Link(_) | ColumnType::File | ColumnType::Member => {
            matches!(operator, Op::Is | Op::IsEmpty | Op::IsNotEmpty)
        }
    }
}

fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

impl RowFilter {
    pub fn compile(&self, columns: &[TableColumn]) -> Result<CompiledFilter, TableError> {
        use FilterOperator as Op;
        let column = column_of(columns, self.column_id)?;
        let column_type = &column.column_type;
        if !allowed(column_type, self.operator) {
            return Err(TableError::InvalidFilter(column.id));
        }
        let bad_value = TableError::InvalidFilterValue(column.id);
        let value = self.value.as_ref().unwrap_or(&Value::Null);
        if value
            .as_array()
            .is_some_and(|items| items.len() > MAX_OPTIONS)
        {
            return Err(bad_value);
        }
        let cell = || {
            CellValue::parse(column_type, value)
                .ok()
                .flatten()
                .map(CellValue::into_json)
                .ok_or(bad_value)
        };
        let comparison = match self.operator {
            Op::Eq | Op::On => Comparison::Eq,
            Op::Ne => Comparison::Ne,
            Op::Lt | Op::Before => Comparison::Lt,
            Op::Le => Comparison::Le,
            Op::Gt | Op::After => Comparison::Gt,
            _ => Comparison::Ge,
        };
        let condition = match (self.operator, column_type) {
            (Op::IsEmpty, _) => RowCondition::Empty,
            (Op::IsNotEmpty, _) => RowCondition::NotEmpty,
            (Op::Checked, _) => RowCondition::Checked(true),
            (Op::Unchecked, _) => RowCondition::Checked(false),
            (Op::Contains, _) => {
                RowCondition::TextContains(cell()?.as_str().unwrap_or_default().to_owned())
            }
            (Op::Is, ColumnType::Text | ColumnType::LongText) => {
                RowCondition::TextIs(cell()?.as_str().unwrap_or_default().to_owned())
            }
            (Op::Is, ColumnType::SingleSelect(_)) => {
                RowCondition::OptionIs(cell()?.as_str().unwrap_or_default().to_owned())
            }
            (Op::IsNot, _) => {
                RowCondition::OptionIsNot(cell()?.as_str().unwrap_or_default().to_owned())
            }
            (Op::AnyOf, ColumnType::SingleSelect(options)) => {
                let multi = ColumnType::MultiSelect(options.clone());
                let ids = CellValue::parse(&multi, value)
                    .ok()
                    .flatten()
                    .ok_or(bad_value)?;
                RowCondition::OptionAnyOf(strings(ids.as_json()))
            }
            (Op::AnyOf, _) => RowCondition::ListAnyOf(strings(&cell()?)),
            (Op::AllOf, _) => RowCondition::ListAllOf(strings(&cell()?)),
            (Op::Is, ColumnType::Link(_)) => RowCondition::LinkIs(
                value
                    .as_str()
                    .and_then(|id| id.parse::<uuid::Uuid>().ok())
                    .ok_or(bad_value)?
                    .to_string(),
            ),
            (Op::Is, _) => RowCondition::IdIs(cell()?.as_str().unwrap_or_default().to_owned()),
            (Op::Before | Op::After | Op::On, _) => RowCondition::DateCompare(
                comparison,
                cell()?
                    .as_str()
                    .and_then(|date| NaiveDate::parse_from_str(date, "%Y-%m-%d").ok())
                    .ok_or(bad_value)?,
            ),
            (_, _) => RowCondition::NumberCompare(comparison, cell()?.to_string()),
        };
        Ok(CompiledFilter {
            column: column.id,
            condition,
        })
    }
}

impl RowSort {
    pub fn compile(&self, columns: &[TableColumn]) -> Result<CompiledSort, TableError> {
        let column = column_of(columns, self.column_id)?;
        let key = match &column.column_type {
            ColumnType::Text | ColumnType::LongText => SortKey::Text,
            ColumnType::Number(_) | ColumnType::Currency => SortKey::Number,
            ColumnType::Date => SortKey::Date,
            ColumnType::Checkbox => SortKey::Checkbox,
            ColumnType::SingleSelect(options) => SortKey::OptionRank(options.ids_in_order()),
            ColumnType::MultiSelect(options) => SortKey::FirstOptionRank(options.ids_in_order()),
            ColumnType::Link(_) | ColumnType::File | ColumnType::Member => {
                return Err(TableError::UnsortableColumn(column.id));
            }
        };
        Ok(CompiledSort {
            column: column.id,
            key,
            direction: self.direction,
        })
    }
}

impl RowQuery {
    pub fn build(
        columns: &[TableColumn],
        filters: &[RowFilter],
        sorts: &[RowSort],
        limit: u64,
        offset: u64,
    ) -> Result<Self, TableError> {
        if filters.len() > MAX_FILTERS {
            return Err(TableError::TooManyFilters);
        }
        if sorts.len() > MAX_SORTS {
            return Err(TableError::TooManySorts);
        }
        Ok(Self {
            filters: filters
                .iter()
                .map(|filter| filter.compile(columns))
                .collect::<Result<_, _>>()?,
            sorts: sorts
                .iter()
                .map(|sort| sort.compile(columns))
                .collect::<Result<_, _>>()?,
            limit: limit.clamp(1, MAX_ROW_PAGE),
            offset: offset.min(MAX_ROWS),
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use common::generate_uuid_v7;
    use serde_json::json;

    use super::*;
    use crate::{
        KnowledgeResourceType, OrganizationId,
        domain::knowledge::tables::{
            KnowledgeTableId, SelectOptionInput, SelectOptions, TableItemName,
            column::{ColumnKind, Decimals},
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

    fn select(labels: &[&str]) -> SelectOptions {
        SelectOptions::parse(
            &labels
                .iter()
                .map(|label| SelectOptionInput {
                    id: None,
                    label: (*label).to_owned(),
                    color: None,
                })
                .collect::<Vec<_>>(),
        )
        .unwrap()
    }

    fn filter(column: &TableColumn, operator: FilterOperator, value: Value) -> RowFilter {
        RowFilter {
            column_id: column.id,
            operator,
            value: Some(value),
        }
    }

    fn all_columns() -> Vec<TableColumn> {
        ColumnKind::ALL
            .into_iter()
            .map(|kind| {
                let config = crate::domain::knowledge::tables::ColumnConfig {
                    link_kind: Some(KnowledgeResourceType::Customer),
                    ..Default::default()
                };
                column(ColumnType::from_config(kind, &config).unwrap())
            })
            .collect()
    }

    #[test]
    fn each_kind_accepts_only_its_operators() {
        use FilterOperator as Op;
        let expected: [(ColumnKind, &[FilterOperator]); 11] = [
            (
                ColumnKind::Text,
                &[Op::Contains, Op::Is, Op::IsEmpty, Op::IsNotEmpty],
            ),
            (
                ColumnKind::LongText,
                &[Op::Contains, Op::Is, Op::IsEmpty, Op::IsNotEmpty],
            ),
            (
                ColumnKind::Number,
                &[
                    Op::Eq,
                    Op::Ne,
                    Op::Lt,
                    Op::Le,
                    Op::Gt,
                    Op::Ge,
                    Op::IsEmpty,
                    Op::IsNotEmpty,
                ],
            ),
            (
                ColumnKind::Currency,
                &[
                    Op::Eq,
                    Op::Ne,
                    Op::Lt,
                    Op::Le,
                    Op::Gt,
                    Op::Ge,
                    Op::IsEmpty,
                    Op::IsNotEmpty,
                ],
            ),
            (
                ColumnKind::Date,
                &[Op::Before, Op::After, Op::On, Op::IsEmpty, Op::IsNotEmpty],
            ),
            (ColumnKind::Checkbox, &[Op::Checked, Op::Unchecked]),
            (
                ColumnKind::SingleSelect,
                &[Op::Is, Op::IsNot, Op::AnyOf, Op::IsEmpty],
            ),
            (
                ColumnKind::MultiSelect,
                &[Op::AnyOf, Op::AllOf, Op::IsEmpty],
            ),
            (ColumnKind::Link, &[Op::Is, Op::IsEmpty, Op::IsNotEmpty]),
            (ColumnKind::File, &[Op::Is, Op::IsEmpty, Op::IsNotEmpty]),
            (ColumnKind::Member, &[Op::Is, Op::IsEmpty, Op::IsNotEmpty]),
        ];
        let every = [
            Op::Contains,
            Op::Is,
            Op::IsNot,
            Op::AnyOf,
            Op::AllOf,
            Op::IsEmpty,
            Op::IsNotEmpty,
            Op::Eq,
            Op::Ne,
            Op::Lt,
            Op::Le,
            Op::Gt,
            Op::Ge,
            Op::Before,
            Op::After,
            Op::On,
            Op::Checked,
            Op::Unchecked,
        ];
        let columns = all_columns();
        for (kind, operators) in expected {
            let column = columns.iter().find(|column| column.kind() == kind).unwrap();
            for operator in every {
                assert_eq!(
                    allowed(&column.column_type, operator),
                    operators.contains(&operator),
                    "{kind:?} {operator:?}"
                );
            }
        }
    }

    #[test]
    fn a_wrong_operator_or_value_is_refused() {
        let amount = column(ColumnType::Currency);
        let date = column(ColumnType::Date);
        let columns = [amount.clone(), date.clone()];
        assert_eq!(
            filter(&amount, FilterOperator::Contains, json!("1")).compile(&columns),
            Err(TableError::InvalidFilter(amount.id))
        );
        assert_eq!(
            filter(&amount, FilterOperator::Gt, json!("mille")).compile(&columns),
            Err(TableError::InvalidFilterValue(amount.id))
        );
        assert_eq!(
            filter(&date, FilterOperator::Before, json!("31/02/2026")).compile(&columns),
            Err(TableError::InvalidFilterValue(date.id))
        );
        let stranger = column(ColumnType::Text);
        assert_eq!(
            filter(&stranger, FilterOperator::Is, json!("x")).compile(&columns),
            Err(TableError::UnknownColumn(stranger.id))
        );
    }

    #[test]
    fn filters_compile_to_typed_conditions() {
        let amount = column(ColumnType::Currency);
        let number = column(ColumnType::Number(Decimals::try_from(2).unwrap()));
        let options = select(&["En cours", "Terminé"]);
        let first = options.as_slice()[0].id.to_string();
        let status = column(ColumnType::SingleSelect(options));
        let date = column(ColumnType::Date);
        let columns = [amount.clone(), number.clone(), status.clone(), date.clone()];
        assert_eq!(
            filter(&amount, FilterOperator::Gt, json!(100_000))
                .compile(&columns)
                .unwrap()
                .condition,
            RowCondition::NumberCompare(Comparison::Gt, "100000".to_owned())
        );
        assert_eq!(
            filter(&number, FilterOperator::Le, json!(2.5))
                .compile(&columns)
                .unwrap()
                .condition,
            RowCondition::NumberCompare(Comparison::Le, "2.5".to_owned())
        );
        assert_eq!(
            filter(&status, FilterOperator::Is, json!(first))
                .compile(&columns)
                .unwrap()
                .condition,
            RowCondition::OptionIs(first.clone())
        );
        assert_eq!(
            filter(&status, FilterOperator::AnyOf, json!([first]))
                .compile(&columns)
                .unwrap()
                .condition,
            RowCondition::OptionAnyOf(vec![first.clone()])
        );
        assert_eq!(
            filter(&status, FilterOperator::Is, json!("En cours")).compile(&columns),
            Err(TableError::InvalidFilterValue(status.id))
        );
        assert_eq!(
            filter(&date, FilterOperator::On, json!("2026-11-05"))
                .compile(&columns)
                .unwrap()
                .condition,
            RowCondition::DateCompare(
                Comparison::Eq,
                NaiveDate::from_ymd_opt(2026, 11, 5).unwrap()
            )
        );
    }

    #[test]
    fn link_file_and_member_columns_cannot_be_sorted_even_in_a_view() {
        for column_type in [
            ColumnType::Link(KnowledgeResourceType::Customer),
            ColumnType::File,
            ColumnType::Member,
        ] {
            let target = column(column_type);
            let sort = RowSort {
                column_id: target.id,
                direction: SortDirection::Asc,
            };
            let columns = std::slice::from_ref(&target);
            assert_eq!(
                sort.compile(columns),
                Err(TableError::UnsortableColumn(target.id))
            );
            let view = crate::domain::knowledge::tables::ViewConfig {
                sorts: vec![sort],
                ..Default::default()
            };
            assert_eq!(
                view.check(crate::domain::knowledge::tables::ViewKind::Grid, columns),
                Err(TableError::UnsortableColumn(target.id))
            );
        }
    }

    #[test]
    fn a_filter_lists_at_most_a_hundred_values() {
        let tags = column(ColumnType::MultiSelect(select(&["A"])));
        let many: Vec<String> = (0..=MAX_OPTIONS)
            .map(|_| generate_uuid_v7().to_string())
            .collect();
        assert_eq!(
            filter(&tags, FilterOperator::AnyOf, json!(many)).compile(std::slice::from_ref(&tags)),
            Err(TableError::InvalidFilterValue(tags.id))
        );
    }

    #[test]
    fn a_query_holds_at_most_three_sorts_and_a_bounded_page() {
        let name = column(ColumnType::Text);
        let columns = [name.clone()];
        let sort = RowSort {
            column_id: name.id,
            direction: SortDirection::Desc,
        };
        assert_eq!(
            RowQuery::build(&columns, &[], &[sort; 4], 50, 0),
            Err(TableError::TooManySorts)
        );
        let query = RowQuery::build(&columns, &[], &[sort], 10_000, 20).unwrap();
        assert_eq!(query.limit, MAX_ROW_PAGE);
        assert_eq!(
            RowQuery::build(&columns, &[], &[], 10, u64::MAX)
                .unwrap()
                .offset,
            MAX_ROWS
        );
        assert_eq!(query.sorts[0].key, SortKey::Text);
    }
}

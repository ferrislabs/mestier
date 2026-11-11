use std::collections::{BTreeMap, HashMap};

use chrono::NaiveDate;
use common::UserId;
use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::{
    KnowledgeArticleId, KnowledgeResource, KnowledgeResourceType,
    domain::knowledge::tables::{
        SelectOptionId, TableColumn, TableColumnId, TableError, column::ColumnType,
    },
};

pub const MISSING_LABEL: &str = "Élément supprimé";

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CellError {
    #[error("expected text")]
    NotText,
    #[error("the text is too long for the column")]
    TextTooLong,
    #[error("expected a number")]
    NotNumber,
    #[error("expected a whole number of cents")]
    NotCents,
    #[error("expected a date written AAAA-MM-JJ")]
    NotDate,
    #[error("expected true or false")]
    NotBoolean,
    #[error("no such option in the column")]
    UnknownOption,
    #[error("an option is listed twice")]
    DuplicateOption,
    #[error("expected a list of option ids")]
    NotOptionList,
    #[error("expected an object with kind and id")]
    NotLink,
    #[error("the link points to another kind of record than the column")]
    WrongLinkKind,
    #[error("expected an id")]
    NotId,
    #[error("not found in the organization")]
    UnknownTarget,
}

impl CellError {
    pub fn code(self) -> &'static str {
        match self {
            CellError::NotText => "not_text",
            CellError::TextTooLong => "text_too_long",
            CellError::NotNumber => "not_number",
            CellError::NotCents => "not_amount",
            CellError::NotDate => "not_date",
            CellError::NotBoolean => "not_boolean",
            CellError::UnknownOption => "unknown_option",
            CellError::DuplicateOption => "duplicate_option",
            CellError::NotOptionList => "not_option_list",
            CellError::NotLink => "not_link",
            CellError::WrongLinkKind => "wrong_link_kind",
            CellError::NotId => "not_id",
            CellError::UnknownTarget => "unknown_target",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellReference {
    Resource(KnowledgeResource),
    File(KnowledgeArticleId),
    Member(UserId),
}

impl CellReference {
    pub fn id(&self) -> Uuid {
        match self {
            CellReference::Resource(resource) => resource.id,
            CellReference::File(id) => id.0,
            CellReference::Member(id) => id.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellValue(Value);

fn uuid_of(raw: &Value) -> Option<Uuid> {
    raw.as_str().and_then(|text| text.parse().ok())
}

fn option_of(raw: &Value, options: &super::SelectOptions) -> Result<SelectOptionId, CellError> {
    uuid_of(raw)
        .map(SelectOptionId)
        .filter(|id| options.contains(*id))
        .ok_or(CellError::UnknownOption)
}

fn text(raw: &Value, limit: usize, trim: bool) -> Result<Option<Value>, CellError> {
    let text = raw.as_str().ok_or(CellError::NotText)?;
    let kept = if trim { text.trim() } else { text };
    if kept.trim().is_empty() {
        return Ok(None);
    }
    if kept.chars().count() > limit {
        return Err(CellError::TextTooLong);
    }
    Ok(Some(Value::String(kept.to_owned())))
}

impl CellValue {
    pub fn parse(column_type: &ColumnType, raw: &Value) -> Result<Option<Self>, CellError> {
        if raw.is_null() {
            return Ok(None);
        }
        let parsed = match column_type {
            ColumnType::Text => text(raw, super::column::MAX_TEXT_CHARS, true)?,
            ColumnType::LongText => text(raw, super::column::MAX_LONG_TEXT_CHARS, false)?,
            ColumnType::Number(_) => match raw {
                Value::Number(number) => Some(Value::Number(number.clone())),
                _ => return Err(CellError::NotNumber),
            },
            ColumnType::Currency => Some(Value::from(raw.as_i64().ok_or(CellError::NotCents)?)),
            ColumnType::Date => {
                let written = raw.as_str().ok_or(CellError::NotDate)?;
                let date = NaiveDate::parse_from_str(written, "%Y-%m-%d")
                    .ok()
                    .filter(|date| date.format("%Y-%m-%d").to_string() == written)
                    .ok_or(CellError::NotDate)?;
                Some(Value::String(date.format("%Y-%m-%d").to_string()))
            }
            ColumnType::Checkbox => Some(Value::Bool(raw.as_bool().ok_or(CellError::NotBoolean)?)),
            ColumnType::SingleSelect(options) => {
                Some(Value::String(option_of(raw, options)?.to_string()))
            }
            ColumnType::MultiSelect(options) => {
                let listed = raw.as_array().ok_or(CellError::NotOptionList)?;
                let mut ids: Vec<SelectOptionId> = Vec::with_capacity(listed.len());
                for item in listed {
                    let id = option_of(item, options)?;
                    if ids.contains(&id) {
                        return Err(CellError::DuplicateOption);
                    }
                    ids.push(id);
                }
                (!ids.is_empty()).then(|| {
                    Value::Array(ids.iter().map(|id| Value::String(id.to_string())).collect())
                })
            }
            ColumnType::Link(expected) => {
                let object = raw.as_object().ok_or(CellError::NotLink)?;
                let kind = object
                    .get("kind")
                    .and_then(Value::as_str)
                    .and_then(|kind| kind.parse::<KnowledgeResourceType>().ok())
                    .ok_or(CellError::NotLink)?;
                if kind != *expected {
                    return Err(CellError::WrongLinkKind);
                }
                let id = object
                    .get("id")
                    .and_then(uuid_of)
                    .ok_or(CellError::NotLink)?;
                Some(json!({ "kind": kind.as_str(), "id": id }))
            }
            ColumnType::File | ColumnType::Member => Some(Value::String(
                uuid_of(raw).ok_or(CellError::NotId)?.to_string(),
            )),
        };
        Ok(parsed.map(CellValue))
    }

    pub fn as_json(&self) -> &Value {
        &self.0
    }

    pub fn into_json(self) -> Value {
        self.0
    }

    pub fn reference(&self, column_type: &ColumnType) -> Option<CellReference> {
        match column_type {
            ColumnType::Link(resource_type) => self.0.get("id").and_then(uuid_of).map(|id| {
                CellReference::Resource(KnowledgeResource {
                    resource_type: *resource_type,
                    id,
                })
            }),
            ColumnType::File => {
                uuid_of(&self.0).map(|id| CellReference::File(KnowledgeArticleId(id)))
            }
            ColumnType::Member => uuid_of(&self.0).map(|id| CellReference::Member(UserId(id))),
            _ => None,
        }
    }

    pub fn display(
        &self,
        column_type: &ColumnType,
        labels: &HashMap<CellReference, String>,
    ) -> String {
        let label_of = |id: &Value| {
            column_type
                .options()
                .zip(uuid_of(id))
                .and_then(|(options, id)| options.label_of(SelectOptionId(id)))
                .unwrap_or_default()
                .to_owned()
        };
        match column_type {
            ColumnType::Text | ColumnType::LongText => {
                self.0.as_str().unwrap_or_default().to_owned()
            }
            ColumnType::Number(_) => decimal_text(&self.0),
            ColumnType::Currency => euros(self.0.as_i64().unwrap_or_default()),
            ColumnType::Date => self
                .0
                .as_str()
                .and_then(|date| NaiveDate::parse_from_str(date, "%Y-%m-%d").ok())
                .map(|date| date.format("%d/%m/%Y").to_string())
                .unwrap_or_default(),
            ColumnType::Checkbox => if self.0.as_bool() == Some(true) {
                "Oui"
            } else {
                "Non"
            }
            .to_owned(),
            ColumnType::SingleSelect(_) => label_of(&self.0),
            ColumnType::MultiSelect(_) => self
                .0
                .as_array()
                .map(|ids| ids.iter().map(label_of).collect::<Vec<_>>().join(", "))
                .unwrap_or_default(),
            ColumnType::Link(_) | ColumnType::File | ColumnType::Member => self
                .reference(column_type)
                .and_then(|reference| labels.get(&reference).cloned())
                .unwrap_or_else(|| MISSING_LABEL.to_owned()),
        }
    }
}

fn decimal_text(number: &Value) -> String {
    match (number.as_i64(), number.as_u64(), number.as_f64()) {
        (Some(whole), _, _) => whole.to_string(),
        (None, Some(whole), _) => whole.to_string(),
        (None, None, Some(real)) => format!("{real}").replace('.', ","),
        (None, None, None) => String::new(),
    }
}

fn euros(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let magnitude = cents.unsigned_abs();
    format!("{sign}{},{:02} €", magnitude / 100, magnitude % 100)
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RowValues(BTreeMap<TableColumnId, CellValue>);

impl RowValues {
    pub fn get(&self, column: TableColumnId) -> Option<&CellValue> {
        self.0.get(&column)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&TableColumnId, &CellValue)> {
        self.0.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn insert(&mut self, column: TableColumnId, cell: CellValue) {
        self.0.insert(column, cell);
    }

    pub fn remove(&mut self, column: TableColumnId) -> Option<CellValue> {
        self.0.remove(&column)
    }

    pub fn to_json(&self) -> Map<String, Value> {
        self.0
            .iter()
            .map(|(column, value)| (column.to_string(), value.as_json().clone()))
            .collect()
    }

    pub fn references(&self, columns: &[TableColumn]) -> Vec<(TableColumnId, CellReference)> {
        columns
            .iter()
            .filter_map(|column| {
                self.0
                    .get(&column.id)
                    .and_then(|value| value.reference(&column.column_type))
                    .map(|reference| (column.id, reference))
            })
            .collect()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RowPatch {
    pub set: RowValues,
    pub cleared: Vec<TableColumnId>,
}

impl RowPatch {
    pub fn parse(
        columns: &[TableColumn],
        raw: &BTreeMap<TableColumnId, Value>,
    ) -> Result<Self, TableError> {
        let mut patch = RowPatch::default();
        for (id, value) in raw {
            let column = columns
                .iter()
                .find(|column| column.id == *id)
                .ok_or(TableError::UnknownColumn(*id))?;
            match CellValue::parse(&column.column_type, value).map_err(|reason| {
                TableError::InvalidCell {
                    column: *id,
                    reason,
                }
            })? {
                Some(cell) => {
                    patch.set.0.insert(*id, cell);
                }
                None => patch.cleared.push(*id),
            }
        }
        Ok(patch)
    }

    pub fn cleared_keys(&self) -> Vec<String> {
        self.cleared.iter().map(ToString::to_string).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.set.is_empty() && self.cleared.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use common::generate_uuid_v7;

    use super::*;
    use crate::{
        OrganizationId,
        domain::knowledge::tables::{
            KnowledgeTableId, SelectOptionInput, SelectOptions, TableItemName,
            column::{Decimals, MAX_LONG_TEXT_CHARS},
        },
    };

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

    fn parsed(column_type: &ColumnType, raw: Value) -> Result<Option<Value>, CellError> {
        CellValue::parse(column_type, &raw).map(|cell| cell.map(CellValue::into_json))
    }

    #[test]
    fn null_clears_a_cell_of_any_kind() {
        for column_type in [
            ColumnType::Text,
            ColumnType::Currency,
            ColumnType::Checkbox,
            ColumnType::Link(KnowledgeResourceType::Quote),
        ] {
            assert_eq!(parsed(&column_type, Value::Null), Ok(None));
        }
    }

    #[test]
    fn text_is_trimmed_bounded_and_blank_is_empty() {
        assert_eq!(
            parsed(&ColumnType::Text, json!("  Villa ")),
            Ok(Some(json!("Villa")))
        );
        assert_eq!(parsed(&ColumnType::Text, json!("   ")), Ok(None));
        assert_eq!(
            parsed(&ColumnType::Text, json!("x".repeat(1_001))),
            Err(CellError::TextTooLong)
        );
        assert_eq!(
            parsed(&ColumnType::Text, json!(12)),
            Err(CellError::NotText)
        );
        assert!(
            parsed(
                &ColumnType::LongText,
                json!("x".repeat(MAX_LONG_TEXT_CHARS))
            )
            .is_ok()
        );
        assert_eq!(
            parsed(
                &ColumnType::LongText,
                json!("x".repeat(MAX_LONG_TEXT_CHARS + 1))
            ),
            Err(CellError::TextTooLong)
        );
    }

    #[test]
    fn a_number_is_a_json_number() {
        let number = ColumnType::Number(Decimals::try_from(2).unwrap());
        assert_eq!(parsed(&number, json!(12.5)), Ok(Some(json!(12.5))));
        assert_eq!(parsed(&number, json!("12")), Err(CellError::NotNumber));
        assert_eq!(parsed(&number, json!("douze")), Err(CellError::NotNumber));
    }

    #[test]
    fn an_amount_is_whole_cents() {
        assert_eq!(
            parsed(&ColumnType::Currency, json!(150_000)),
            Ok(Some(json!(150_000)))
        );
        assert_eq!(
            parsed(&ColumnType::Currency, json!(12.5)),
            Err(CellError::NotCents)
        );
        assert_eq!(
            parsed(&ColumnType::Currency, json!("1200")),
            Err(CellError::NotCents)
        );
    }

    #[test]
    fn a_date_is_a_real_iso_day() {
        assert_eq!(
            parsed(&ColumnType::Date, json!("2026-11-05")),
            Ok(Some(json!("2026-11-05")))
        );
        for wrong in [
            "2026-02-31",
            "31/02/2026",
            "2026-2-3",
            "2026-11-05T10:00:00",
            "",
        ] {
            assert_eq!(
                parsed(&ColumnType::Date, json!(wrong)),
                Err(CellError::NotDate),
                "{wrong}"
            );
        }
    }

    #[test]
    fn a_checkbox_is_a_boolean() {
        assert_eq!(
            parsed(&ColumnType::Checkbox, json!(false)),
            Ok(Some(json!(false)))
        );
        assert_eq!(
            parsed(&ColumnType::Checkbox, json!("oui")),
            Err(CellError::NotBoolean)
        );
    }

    #[test]
    fn a_choice_names_an_existing_option() {
        let options = select(&["En cours", "Terminé"]);
        let first = options.as_slice()[0].id.to_string();
        let second = options.as_slice()[1].id.to_string();
        let single = ColumnType::SingleSelect(options.clone());
        assert_eq!(parsed(&single, json!(first)), Ok(Some(json!(first))));
        assert_eq!(
            parsed(&single, json!(generate_uuid_v7().to_string())),
            Err(CellError::UnknownOption)
        );
        assert_eq!(
            parsed(&single, json!("En cours")),
            Err(CellError::UnknownOption)
        );

        let multi = ColumnType::MultiSelect(options);
        assert_eq!(
            parsed(&multi, json!([first, second])),
            Ok(Some(json!([first, second])))
        );
        assert_eq!(parsed(&multi, json!([])), Ok(None));
        assert_eq!(
            parsed(&multi, json!([first, first])),
            Err(CellError::DuplicateOption)
        );
        assert_eq!(parsed(&multi, json!(first)), Err(CellError::NotOptionList));
    }

    #[test]
    fn a_link_matches_the_kind_of_its_column() {
        let link = ColumnType::Link(KnowledgeResourceType::Customer);
        let id = generate_uuid_v7();
        assert_eq!(
            parsed(&link, json!({"kind": "customer", "id": id})),
            Ok(Some(json!({"kind": "customer", "id": id})))
        );
        assert_eq!(
            parsed(&link, json!({"kind": "project", "id": id})),
            Err(CellError::WrongLinkKind)
        );
        assert_eq!(parsed(&link, json!(id)), Err(CellError::NotLink));
        assert_eq!(
            parsed(&link, json!({"kind": "customer", "id": "x"})),
            Err(CellError::NotLink)
        );
    }

    #[test]
    fn a_file_and_a_member_are_ids() {
        let id = generate_uuid_v7();
        assert_eq!(parsed(&ColumnType::File, json!(id)), Ok(Some(json!(id))));
        assert_eq!(
            parsed(&ColumnType::Member, json!("moi")),
            Err(CellError::NotId)
        );
        let cell = CellValue::parse(&ColumnType::Member, &json!(id))
            .unwrap()
            .unwrap();
        assert_eq!(
            cell.reference(&ColumnType::Member),
            Some(CellReference::Member(UserId(id)))
        );
    }

    #[test]
    fn a_number_displays_in_full_without_exponent_nor_trailing_zero() {
        let number = ColumnType::Number(Decimals::try_from(2).unwrap());
        let shown = |raw: &str| {
            let raw: Value = serde_json::from_str(raw).unwrap();
            CellValue::parse(&number, &raw)
                .unwrap()
                .unwrap()
                .display(&number, &HashMap::new())
        };
        assert_eq!(shown("12.0"), "12");
        assert_eq!(shown("1e21"), "1000000000000000000000");
        assert_eq!(shown("0.00001"), "0,00001");
        assert_eq!(shown("1234.5"), "1234,5");
        assert_eq!(shown("-7"), "-7");
    }

    #[test]
    fn values_display_as_people_read_them() {
        let labels = HashMap::new();
        let shown = |column_type: &ColumnType, raw: Value| {
            CellValue::parse(column_type, &raw)
                .unwrap()
                .unwrap()
                .display(column_type, &labels)
        };
        assert_eq!(shown(&ColumnType::Currency, json!(150_050)), "1500,50 €");
        assert_eq!(shown(&ColumnType::Currency, json!(-5)), "-0,05 €");
        assert_eq!(shown(&ColumnType::Date, json!("2026-11-05")), "05/11/2026");
        assert_eq!(shown(&ColumnType::Checkbox, json!(true)), "Oui");
        assert_eq!(
            shown(&ColumnType::File, json!(generate_uuid_v7())),
            MISSING_LABEL
        );
    }

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

    #[test]
    fn a_patch_sets_clears_and_refuses_unknown_columns() {
        let name = column(ColumnType::Text);
        let done = column(ColumnType::Checkbox);
        let columns = [name.clone(), done.clone()];
        let raw = BTreeMap::from([(name.id, json!("Villa")), (done.id, Value::Null)]);
        let patch = RowPatch::parse(&columns, &raw).unwrap();
        assert_eq!(
            patch.set.to_json(),
            Map::from_iter([(name.key(), json!("Villa"))])
        );
        assert_eq!(patch.cleared, vec![done.id]);

        let stranger = TableColumnId(generate_uuid_v7());
        assert_eq!(
            RowPatch::parse(&columns, &BTreeMap::from([(stranger, json!("x"))])),
            Err(TableError::UnknownColumn(stranger))
        );
        assert_eq!(
            RowPatch::parse(&columns, &BTreeMap::from([(done.id, json!("x"))])),
            Err(TableError::InvalidCell {
                column: done.id,
                reason: CellError::NotBoolean
            })
        );
    }
}

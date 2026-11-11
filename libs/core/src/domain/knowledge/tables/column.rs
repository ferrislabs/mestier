use std::{collections::HashMap, str::FromStr};

use common::generate_uuid_v7;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use crate::{
    KnowledgeResourceType,
    domain::knowledge::tables::{
        MAX_OPTIONS, SelectOptionId, TableError, TableRowId,
        value::{CellReference, CellValue},
    },
};

pub const MAX_TEXT_CHARS: usize = 1_000;
pub const MAX_LONG_TEXT_CHARS: usize = 20_000;
pub const MAX_OPTION_LABEL_CHARS: usize = 100;
pub const MAX_DECIMALS: u8 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ColumnKind {
    Text,
    LongText,
    Number,
    Currency,
    Date,
    Checkbox,
    SingleSelect,
    MultiSelect,
    Link,
    File,
    Member,
}

impl ColumnKind {
    pub const ALL: [ColumnKind; 11] = [
        ColumnKind::Text,
        ColumnKind::LongText,
        ColumnKind::Number,
        ColumnKind::Currency,
        ColumnKind::Date,
        ColumnKind::Checkbox,
        ColumnKind::SingleSelect,
        ColumnKind::MultiSelect,
        ColumnKind::Link,
        ColumnKind::File,
        ColumnKind::Member,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            ColumnKind::Text => "text",
            ColumnKind::LongText => "long_text",
            ColumnKind::Number => "number",
            ColumnKind::Currency => "currency",
            ColumnKind::Date => "date",
            ColumnKind::Checkbox => "checkbox",
            ColumnKind::SingleSelect => "single_select",
            ColumnKind::MultiSelect => "multi_select",
            ColumnKind::Link => "link",
            ColumnKind::File => "file",
            ColumnKind::Member => "member",
        }
    }
}

impl ColumnKind {
    pub fn is_reference(self) -> bool {
        matches!(
            self,
            ColumnKind::Link | ColumnKind::File | ColumnKind::Member
        )
    }
}

impl FromStr for ColumnKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ColumnKind::ALL
            .into_iter()
            .find(|kind| kind.as_str() == s)
            .ok_or(())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SelectColor {
    #[default]
    Gray,
    Red,
    Orange,
    Yellow,
    Green,
    Teal,
    Blue,
    Purple,
    Pink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Decimals(u8);

impl Decimals {
    pub fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for Decimals {
    type Error = TableError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > MAX_DECIMALS {
            return Err(TableError::DecimalsOutOfRange);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectOption {
    pub id: SelectOptionId,
    pub label: String,
    pub color: SelectColor,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SelectOptions(Vec<SelectOption>);

impl SelectOptions {
    pub fn parse(inputs: &[SelectOptionInput]) -> Result<Self, TableError> {
        if inputs.len() > MAX_OPTIONS {
            return Err(TableError::TooManyOptions);
        }
        let mut options: Vec<SelectOption> = Vec::with_capacity(inputs.len());
        for input in inputs {
            let label = input.label.trim();
            if label.is_empty() {
                return Err(TableError::OptionLabelEmpty);
            }
            if label.chars().count() > MAX_OPTION_LABEL_CHARS {
                return Err(TableError::OptionLabelTooLong);
            }
            let lowered = label.to_lowercase();
            if options
                .iter()
                .any(|option| option.label.to_lowercase() == lowered)
            {
                return Err(TableError::OptionLabelTaken);
            }
            let id = input
                .id
                .unwrap_or_else(|| SelectOptionId(generate_uuid_v7()));
            if options.iter().any(|option| option.id == id) {
                return Err(TableError::OptionIdTaken);
            }
            options.push(SelectOption {
                id,
                label: label.to_owned(),
                color: input.color.unwrap_or_default(),
            });
        }
        Ok(Self(options))
    }

    pub fn as_slice(&self) -> &[SelectOption] {
        &self.0
    }

    pub fn contains(&self, id: SelectOptionId) -> bool {
        self.0.iter().any(|option| option.id == id)
    }

    pub fn label_of(&self, id: SelectOptionId) -> Option<&str> {
        self.0
            .iter()
            .find(|option| option.id == id)
            .map(|option| option.label.as_str())
    }

    pub fn labelled(&self, label: &str) -> Option<&SelectOption> {
        let wanted = label.trim().to_lowercase();
        self.0
            .iter()
            .find(|option| option.label.to_lowercase() == wanted)
    }

    pub fn resolve(&self, written: &str) -> Option<SelectOptionId> {
        self.labelled(written).map(|option| option.id).or_else(|| {
            written
                .trim()
                .parse()
                .ok()
                .map(SelectOptionId)
                .filter(|id| self.contains(*id))
        })
    }

    pub fn ids_in_order(&self) -> Vec<String> {
        self.0.iter().map(|option| option.id.to_string()).collect()
    }

    pub fn removed_from(&self, before: &SelectOptions) -> Vec<SelectOptionId> {
        before
            .0
            .iter()
            .filter(|option| !self.contains(option.id))
            .map(|option| option.id)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SelectOptionInput {
    #[serde(default)]
    pub id: Option<SelectOptionId>,
    pub label: String,
    #[serde(default)]
    pub color: Option<SelectColor>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ColumnConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decimals: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<SelectOptionInput>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_kind: Option<KnowledgeResourceType>,
}

impl ColumnConfig {
    pub fn reusing_ids(mut self, existing: Option<&SelectOptions>) -> Self {
        let Some(existing) = existing else {
            return self;
        };
        for input in self.options.iter_mut().flatten() {
            if input.id.is_none() {
                input.id = existing
                    .as_slice()
                    .iter()
                    .find(|option| {
                        option.label.trim().to_lowercase() == input.label.trim().to_lowercase()
                    })
                    .map(|option| option.id);
            }
        }
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnType {
    Text,
    LongText,
    Number(Decimals),
    Currency,
    Date,
    Checkbox,
    SingleSelect(SelectOptions),
    MultiSelect(SelectOptions),
    Link(KnowledgeResourceType),
    File,
    Member,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Conversion {
    Unchanged,
    Bounded { limit: usize },
    ToText { limit: usize },
    SingleToMulti,
    MultiToSingle,
}

impl ColumnType {
    pub fn from_config(kind: ColumnKind, config: &ColumnConfig) -> Result<Self, TableError> {
        let options = || SelectOptions::parse(config.options.as_deref().unwrap_or_default());
        Ok(match kind {
            ColumnKind::Text => ColumnType::Text,
            ColumnKind::LongText => ColumnType::LongText,
            ColumnKind::Number => {
                ColumnType::Number(Decimals::try_from(config.decimals.unwrap_or_default())?)
            }
            ColumnKind::Currency => ColumnType::Currency,
            ColumnKind::Date => ColumnType::Date,
            ColumnKind::Checkbox => ColumnType::Checkbox,
            ColumnKind::SingleSelect => ColumnType::SingleSelect(options()?),
            ColumnKind::MultiSelect => ColumnType::MultiSelect(options()?),
            ColumnKind::Link => {
                ColumnType::Link(config.link_kind.ok_or(TableError::LinkKindMissing)?)
            }
            ColumnKind::File => ColumnType::File,
            ColumnKind::Member => ColumnType::Member,
        })
    }

    pub fn kind(&self) -> ColumnKind {
        match self {
            ColumnType::Text => ColumnKind::Text,
            ColumnType::LongText => ColumnKind::LongText,
            ColumnType::Number(_) => ColumnKind::Number,
            ColumnType::Currency => ColumnKind::Currency,
            ColumnType::Date => ColumnKind::Date,
            ColumnType::Checkbox => ColumnKind::Checkbox,
            ColumnType::SingleSelect(_) => ColumnKind::SingleSelect,
            ColumnType::MultiSelect(_) => ColumnKind::MultiSelect,
            ColumnType::Link(_) => ColumnKind::Link,
            ColumnType::File => ColumnKind::File,
            ColumnType::Member => ColumnKind::Member,
        }
    }

    pub fn config(&self) -> ColumnConfig {
        match self {
            ColumnType::Number(decimals) => ColumnConfig {
                decimals: Some(decimals.get()),
                ..ColumnConfig::default()
            },
            ColumnType::SingleSelect(options) | ColumnType::MultiSelect(options) => ColumnConfig {
                options: Some(
                    options
                        .as_slice()
                        .iter()
                        .map(|option| SelectOptionInput {
                            id: Some(option.id),
                            label: option.label.clone(),
                            color: Some(option.color),
                        })
                        .collect(),
                ),
                ..ColumnConfig::default()
            },
            ColumnType::Link(kind) => ColumnConfig {
                link_kind: Some(*kind),
                ..ColumnConfig::default()
            },
            _ => ColumnConfig::default(),
        }
    }

    pub fn options(&self) -> Option<&SelectOptions> {
        match self {
            ColumnType::SingleSelect(options) | ColumnType::MultiSelect(options) => Some(options),
            _ => None,
        }
    }

    pub fn text_limit(&self) -> Option<usize> {
        match self {
            ColumnType::Text => Some(MAX_TEXT_CHARS),
            ColumnType::LongText => Some(MAX_LONG_TEXT_CHARS),
            _ => None,
        }
    }

    pub fn conversion_to(&self, target: &ColumnType) -> Result<Conversion, TableError> {
        match (self, target) {
            (ColumnType::Link(from), ColumnType::Link(to)) if from != to => {
                Err(TableError::ConversionRefused)
            }
            (from, to) if from.kind() == to.kind() => Ok(Conversion::Unchanged),
            (from, to) => match (from.text_limit(), to.text_limit()) {
                (Some(_), Some(limit)) => Ok(Conversion::Bounded { limit }),
                (None, Some(limit)) => Ok(Conversion::ToText { limit }),
                _ => match (from, to) {
                    (ColumnType::SingleSelect(_), ColumnType::MultiSelect(_)) => {
                        Ok(Conversion::SingleToMulti)
                    }
                    (ColumnType::MultiSelect(_), ColumnType::SingleSelect(_)) => {
                        Ok(Conversion::MultiToSingle)
                    }
                    _ => Err(TableError::ConversionRefused),
                },
            },
        }
    }

    pub fn convert_stored(
        &self,
        conversion: Conversion,
        stored: &[(TableRowId, Value)],
        labels: &HashMap<CellReference, String>,
    ) -> Result<Vec<(TableRowId, Value)>, TableError> {
        let mut rewritten = Vec::new();
        for (id, value) in stored {
            match conversion {
                Conversion::Unchanged => {}
                Conversion::Bounded { limit } => {
                    if value
                        .as_str()
                        .is_some_and(|text| text.chars().count() > limit)
                    {
                        return Err(TableError::ConversionLosesValue);
                    }
                }
                Conversion::ToText { limit } => {
                    let Ok(Some(cell)) = CellValue::parse(self, value) else {
                        return Err(TableError::ConversionLosesValue);
                    };
                    let text = cell.display(self, labels);
                    if text.chars().count() > limit {
                        return Err(TableError::ConversionLosesValue);
                    }
                    rewritten.push((*id, Value::String(text)));
                }
                Conversion::SingleToMulti => {
                    rewritten.push((*id, Value::Array(vec![value.clone()])));
                }
                Conversion::MultiToSingle => match value.as_array().map(Vec::as_slice) {
                    Some([single]) => rewritten.push((*id, single.clone())),
                    _ => return Err(TableError::ConversionLosesValue),
                },
            }
        }
        Ok(rewritten)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn option(label: &str) -> SelectOptionInput {
        SelectOptionInput {
            id: None,
            label: label.to_owned(),
            color: None,
        }
    }

    fn select(labels: &[&str]) -> SelectOptions {
        SelectOptions::parse(&labels.iter().map(|label| option(label)).collect::<Vec<_>>()).unwrap()
    }

    fn row() -> TableRowId {
        TableRowId(generate_uuid_v7())
    }

    #[test]
    fn every_kind_reads_back_from_its_name() {
        for kind in ColumnKind::ALL {
            assert_eq!(kind.as_str().parse::<ColumnKind>(), Ok(kind));
        }
        assert!("formula".parse::<ColumnKind>().is_err());
    }

    #[test]
    fn a_config_round_trips_through_its_type() {
        let config = ColumnConfig {
            options: Some(vec![option("En cours"), option("Terminé")]),
            ..ColumnConfig::default()
        };
        let column = ColumnType::from_config(ColumnKind::SingleSelect, &config).unwrap();
        let stored = column.config();
        assert_eq!(
            ColumnType::from_config(ColumnKind::SingleSelect, &stored).unwrap(),
            column
        );
    }

    #[test]
    fn decimals_are_bounded() {
        let config = |decimals| ColumnConfig {
            decimals: Some(decimals),
            ..ColumnConfig::default()
        };
        assert!(ColumnType::from_config(ColumnKind::Number, &config(4)).is_ok());
        assert_eq!(
            ColumnType::from_config(ColumnKind::Number, &config(5)),
            Err(TableError::DecimalsOutOfRange)
        );
    }

    #[test]
    fn a_link_column_names_its_target() {
        assert_eq!(
            ColumnType::from_config(ColumnKind::Link, &ColumnConfig::default()),
            Err(TableError::LinkKindMissing)
        );
    }

    #[test]
    fn options_are_bounded_unique_and_labelled() {
        let many: Vec<_> = (0..101).map(|n| option(&format!("o{n}"))).collect();
        assert_eq!(SelectOptions::parse(&many), Err(TableError::TooManyOptions));
        assert_eq!(
            SelectOptions::parse(&[option("A"), option("a")]),
            Err(TableError::OptionLabelTaken)
        );
        assert_eq!(
            SelectOptions::parse(&[option(" ")]),
            Err(TableError::OptionLabelEmpty)
        );
        let id = SelectOptionId(generate_uuid_v7());
        let twice = [
            SelectOptionInput {
                id: Some(id),
                ..option("A")
            },
            SelectOptionInput {
                id: Some(id),
                ..option("B")
            },
        ];
        assert_eq!(SelectOptions::parse(&twice), Err(TableError::OptionIdTaken));
    }

    #[test]
    fn an_option_is_found_by_its_label_without_case_or_by_its_id() {
        let options = select(&["En cours", "Terminé"]);
        let ongoing = options.as_slice()[0].id;
        let done = options.as_slice()[1].id;
        assert_eq!(
            options.labelled(" en COURS ").map(|option| option.id),
            Some(ongoing)
        );
        assert_eq!(options.resolve("TERMINÉ"), Some(done));
        assert_eq!(options.resolve(&done.to_string()), Some(done));
        assert_eq!(options.resolve("Annulé"), None);
        assert_eq!(
            options.resolve(&SelectOptionId(generate_uuid_v7()).to_string()),
            None
        );
    }

    #[test]
    fn only_lossless_conversions_are_allowed() {
        let number = ColumnType::Number(Decimals::try_from(2).unwrap());
        assert_eq!(
            number.conversion_to(&ColumnType::Text),
            Ok(Conversion::ToText {
                limit: MAX_TEXT_CHARS
            })
        );
        assert_eq!(
            ColumnType::Text.conversion_to(&ColumnType::LongText),
            Ok(Conversion::Bounded {
                limit: MAX_LONG_TEXT_CHARS
            })
        );
        assert_eq!(
            ColumnType::LongText.conversion_to(&ColumnType::Text),
            Ok(Conversion::Bounded {
                limit: MAX_TEXT_CHARS
            })
        );
        assert_eq!(
            ColumnType::SingleSelect(select(&["A"]))
                .conversion_to(&ColumnType::MultiSelect(select(&["A"]))),
            Ok(Conversion::SingleToMulti)
        );
        assert_eq!(
            ColumnType::MultiSelect(select(&["A"]))
                .conversion_to(&ColumnType::SingleSelect(select(&["A"]))),
            Ok(Conversion::MultiToSingle)
        );
        assert_eq!(
            number.conversion_to(&ColumnType::Date),
            Err(TableError::ConversionRefused)
        );
        assert_eq!(
            ColumnType::Text.conversion_to(&ColumnType::Number(Decimals::try_from(0).unwrap())),
            Err(TableError::ConversionRefused)
        );
        assert_eq!(
            ColumnType::Link(KnowledgeResourceType::Customer)
                .conversion_to(&ColumnType::Link(KnowledgeResourceType::Project)),
            Err(TableError::ConversionRefused)
        );
        assert_eq!(
            ColumnType::Checkbox.conversion_to(&ColumnType::Checkbox),
            Ok(Conversion::Unchanged)
        );
    }

    #[test]
    fn a_number_becomes_its_text() {
        let number = ColumnType::Number(Decimals::try_from(0).unwrap());
        let id = row();
        let rewritten = number
            .convert_stored(
                Conversion::ToText {
                    limit: MAX_TEXT_CHARS,
                },
                &[(id, json!(1250.5))],
                &HashMap::new(),
            )
            .unwrap();
        assert_eq!(rewritten, vec![(id, json!("1250,5"))]);
    }

    #[test]
    fn a_long_text_too_long_for_text_is_refused() {
        let stored = [(row(), json!("x".repeat(1_001)))];
        assert_eq!(
            ColumnType::LongText.convert_stored(
                Conversion::Bounded {
                    limit: MAX_TEXT_CHARS
                },
                &stored,
                &HashMap::new()
            ),
            Err(TableError::ConversionLosesValue)
        );
    }

    #[test]
    fn a_multiple_choice_with_two_options_cannot_become_single() {
        let options = select(&["A", "B"]);
        let [a, b] = [options.as_slice()[0].id, options.as_slice()[1].id];
        let multi = ColumnType::MultiSelect(options);
        let one = row();
        assert_eq!(
            multi.convert_stored(
                Conversion::MultiToSingle,
                &[(one, json!([a.to_string()]))],
                &HashMap::new()
            ),
            Ok(vec![(one, json!(a.to_string()))])
        );
        assert_eq!(
            multi.convert_stored(
                Conversion::MultiToSingle,
                &[(row(), json!([a.to_string(), b.to_string()]))],
                &HashMap::new()
            ),
            Err(TableError::ConversionLosesValue)
        );
    }

    #[test]
    fn a_choice_becomes_its_label_and_a_link_its_name() {
        let options = select(&["En cours"]);
        let id = options.as_slice()[0].id;
        let single = ColumnType::SingleSelect(options);
        let converted = single
            .convert_stored(
                Conversion::ToText {
                    limit: MAX_TEXT_CHARS,
                },
                &[(row(), json!(id.to_string()))],
                &HashMap::new(),
            )
            .unwrap();
        assert_eq!(converted[0].1, json!("En cours"));

        let customer = generate_uuid_v7();
        let link = ColumnType::Link(KnowledgeResourceType::Customer);
        let labels = HashMap::from([(
            CellReference::Resource(crate::KnowledgeResource {
                resource_type: KnowledgeResourceType::Customer,
                id: customer,
            }),
            "Dupont SARL".to_owned(),
        )]);
        let converted = link
            .convert_stored(
                Conversion::ToText {
                    limit: MAX_TEXT_CHARS,
                },
                &[(row(), json!({"kind": "customer", "id": customer}))],
                &labels,
            )
            .unwrap();
        assert_eq!(converted[0].1, json!("Dupont SARL"));
    }
}

use events::{DomainEvent, EventDescriptor, EventSubject};
use serde_json::{Map, Value, json};

use crate::domain::knowledge::tables::{KnowledgeTable, ResolvedRow, TableColumn};

pub struct TableRowCreated<'a> {
    pub table: &'a KnowledgeTable,
    pub columns: &'a [TableColumn],
    pub row: &'a ResolvedRow,
}

impl DomainEvent for TableRowCreated<'_> {
    fn name(&self) -> &'static str {
        "table_row.created"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("table_row", self.row.row.id.0)
    }

    fn payload(&self) -> Value {
        json!({
            "table": table_payload(self.table),
            "row": row_payload(self.columns, self.row),
        })
    }
}

pub struct TableRowUpdated<'a> {
    pub table: &'a KnowledgeTable,
    pub columns: &'a [TableColumn],
    pub row: &'a ResolvedRow,
    pub changed: Vec<&'a str>,
}

impl DomainEvent for TableRowUpdated<'_> {
    fn name(&self) -> &'static str {
        "table_row.updated"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("table_row", self.row.row.id.0)
    }

    fn payload(&self) -> Value {
        json!({
            "table": table_payload(self.table),
            "row": row_payload(self.columns, self.row),
            "changed": self.changed,
        })
    }
}

fn table_payload(table: &KnowledgeTable) -> Value {
    json!({ "id": table.id.0, "name": table.name.as_str() })
}

pub fn row_payload(columns: &[TableColumn], row: &ResolvedRow) -> Value {
    json!({
        "id": row.row.id.0,
        "title": row.row.title(columns),
        "values": row.row.values,
        "fields": fields(columns, row),
    })
}

fn fields(columns: &[TableColumn], row: &ResolvedRow) -> Map<String, Value> {
    columns
        .iter()
        .map(|column| {
            (
                column.name.as_str().to_owned(),
                row.shown(column).map_or(Value::Null, Value::String),
            )
        })
        .collect()
}

pub fn changed_columns<'a>(
    columns: &'a [TableColumn],
    before: &Map<String, Value>,
    after: &Map<String, Value>,
) -> Vec<&'a str> {
    columns
        .iter()
        .filter(|column| {
            let key = column.key();
            before.get(&key) != after.get(&key)
        })
        .map(|column| column.name.as_str())
        .collect()
}

#[cfg(test)]
pub fn emitted_events() -> Vec<(&'static str, u16)> {
    vec![("table_row.created", 1), ("table_row.updated", 1)]
}

pub fn descriptors() -> Vec<EventDescriptor> {
    let table = json!({
        "id": "018f3b2a-0000-7000-8000-000000000001",
        "name": "Chantiers",
    });
    let row = json!({
        "id": "018f3b2a-0000-7000-8000-000000000002",
        "title": "Villa Dupont",
        "values": {
            "018f3b2a-0000-7000-8000-000000000003": "Villa Dupont",
            "018f3b2a-0000-7000-8000-000000000004": "018f3b2a-0000-7000-8000-000000000005",
            "018f3b2a-0000-7000-8000-000000000006": true,
        },
        "fields": {
            "Nom": "Villa Dupont",
            "Statut": "En cours",
            "Traité": "Oui",
            "Note": null,
        },
    });

    vec![
        EventDescriptor {
            name: "table_row.created",
            version: 1,
            label: "Ligne de table créée",
            subject_kind: "table_row",
            payload_example: json!({ "table": table, "row": row }),
        },
        EventDescriptor {
            name: "table_row.updated",
            version: 1,
            label: "Ligne de table modifiée",
            subject_kind: "table_row",
            payload_example: json!({
                "table": table,
                "row": row,
                "changed": ["Statut", "Traité"],
            }),
        },
    ]
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use chrono::Utc;
    use common::generate_uuid_v7;
    use serde_json::json;

    use super::*;
    use crate::{
        KnowledgeResourceType, OrganizationId,
        domain::knowledge::tables::{
            CellLabel, KnowledgeTableId, SelectOptionInput, SelectOptions, TableColumnId,
            TableItemName, TableRow, TableRowId,
            column::{ColumnType, Decimals},
        },
    };

    fn column(name: &str, column_type: ColumnType, is_primary: bool) -> TableColumn {
        TableColumn {
            id: TableColumnId(generate_uuid_v7()),
            table_id: KnowledgeTableId(generate_uuid_v7()),
            organization_id: OrganizationId(generate_uuid_v7()),
            name: TableItemName::try_from(name).unwrap(),
            column_type,
            position: 0,
            is_primary,
            created_at: Utc::now(),
        }
    }

    fn table() -> KnowledgeTable {
        KnowledgeTable {
            id: KnowledgeTableId(generate_uuid_v7()),
            organization_id: OrganizationId(generate_uuid_v7()),
            name: TableItemName::try_from("Chantiers").unwrap(),
            description: None,
            created_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    struct Fixture {
        table: KnowledgeTable,
        columns: Vec<TableColumn>,
        row: ResolvedRow,
    }

    fn fixture() -> Fixture {
        let status = SelectOptions::parse(&[SelectOptionInput {
            id: None,
            label: "En cours".to_owned(),
            color: None,
        }])
        .unwrap();
        let ongoing = status.as_slice()[0].id.to_string();
        let name = column("Nom", ColumnType::Text, true);
        let amount = column("Montant", ColumnType::Currency, false);
        let state = column("Statut", ColumnType::SingleSelect(status), false);
        let done = column("Traité", ColumnType::Checkbox, false);
        let client = column(
            "Client",
            ColumnType::Link(KnowledgeResourceType::Customer),
            false,
        );
        let surface = column(
            "Surface",
            ColumnType::Number(Decimals::try_from(1).unwrap()),
            false,
        );
        let values = Map::from_iter([
            (name.key(), json!("Villa Dupont")),
            (amount.key(), json!(125_050)),
            (state.key(), json!(ongoing)),
            (done.key(), json!(false)),
            (
                client.key(),
                json!({"kind": "customer", "id": generate_uuid_v7()}),
            ),
        ]);
        let row = ResolvedRow {
            row: TableRow {
                id: TableRowId(generate_uuid_v7()),
                table_id: KnowledgeTableId(generate_uuid_v7()),
                organization_id: OrganizationId(generate_uuid_v7()),
                values,
                created_by: None,
                updated_by: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            labels: BTreeMap::from([(client.id, CellLabel::Found("Dupont SARL".to_owned()))]),
        };
        Fixture {
            table: table(),
            columns: vec![name, amount, state, done, client, surface],
            row,
        }
    }

    #[test]
    fn a_created_row_names_its_table_its_title_and_its_fields_as_displayed() {
        let fixture = fixture();
        let event = TableRowCreated {
            table: &fixture.table,
            columns: &fixture.columns,
            row: &fixture.row,
        };

        let payload = event.payload();

        assert_eq!(
            payload["table"],
            json!({"id": fixture.table.id.0, "name": "Chantiers"})
        );
        assert_eq!(payload["row"]["id"], json!(fixture.row.row.id.0));
        assert_eq!(payload["row"]["title"], json!("Villa Dupont"));
        assert_eq!(
            payload["row"]["values"],
            Value::Object(fixture.row.row.values.clone())
        );
        assert_eq!(
            payload["row"]["fields"],
            json!({
                "Nom": "Villa Dupont",
                "Montant": "1250,50 €",
                "Statut": "En cours",
                "Traité": "Non",
                "Client": "Dupont SARL",
                "Surface": null,
            })
        );
        assert!(payload.get("changed").is_none());
        assert_eq!(
            event.subject(),
            EventSubject::new("table_row", fixture.row.row.id.0)
        );
    }

    #[test]
    fn an_updated_row_lists_the_names_of_the_columns_that_changed() {
        let fixture = fixture();
        let event = TableRowUpdated {
            table: &fixture.table,
            columns: &fixture.columns,
            row: &fixture.row,
            changed: vec!["Statut", "Traité"],
        };

        let payload = event.payload();

        assert_eq!(payload["changed"], json!(["Statut", "Traité"]));
        assert_eq!(payload["row"]["title"], json!("Villa Dupont"));
        assert_eq!(payload["table"]["name"], json!("Chantiers"));
    }

    #[test]
    fn a_row_without_a_title_says_so() {
        let fixture = fixture();
        let mut row = fixture.row.clone();
        row.row.values.remove(&fixture.columns[0].key());

        assert_eq!(row_payload(&fixture.columns, &row)["title"], Value::Null);
    }

    #[test]
    fn changed_columns_are_the_ones_whose_value_moved_set_or_cleared_in_column_order() {
        let fixture = fixture();
        let before = fixture.row.row.values.clone();
        let mut after = before.clone();
        after.insert(fixture.columns[3].key(), json!(true));
        after.remove(&fixture.columns[1].key());
        after.insert(fixture.columns[5].key(), json!(12.5));
        after.insert(fixture.columns[0].key(), json!("Villa Dupont"));

        assert_eq!(
            changed_columns(&fixture.columns, &before, &after),
            vec!["Montant", "Traité", "Surface"]
        );
        assert!(changed_columns(&fixture.columns, &before, &before).is_empty());
    }

    #[test]
    fn every_emitted_event_is_described_with_a_french_label_and_an_example() {
        let described: Vec<(&str, u16)> = descriptors()
            .iter()
            .map(|descriptor| (descriptor.name, descriptor.version))
            .collect();

        assert_eq!(described, emitted_events());
        for descriptor in descriptors() {
            assert_eq!(descriptor.subject_kind, "table_row");
            assert!(descriptor.payload_example["row"]["fields"].is_object());
            assert_eq!(
                descriptor.payload_example["row"]["fields"].get("Note"),
                Some(&Value::Null)
            );
            assert!(descriptor.label.starts_with("Ligne de table"));
        }
        assert!(descriptors()[1].payload_example["changed"].is_array());
    }
}
